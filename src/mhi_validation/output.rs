//! Deterministic Phase-E publication bundle.
//!
//! The writer deliberately projects the already validated typed report. It
//! never recomputes scientific results, and it keeps operational details out
//! of managed bytes so a bundle is reproducible on another host.

use super::MhiValidationError;
use super::error::{PublicationFingerprintResult, PublicationIdentityResult, PublicationPathState};
use crate::{
    domain::{read_artifact_strict_bytes, serialize_artifact},
    mhi_validation::statistics::MetricValueV1,
    mhi_validation::{MhiValidationProtocolV1, ValidationInputs},
    results::{DatasetSourceReferenceV1, MhiValidationReportV1},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[cfg(not(unix))]
use std::fs::OpenOptions;
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[cfg(unix)]
use std::{
    ffi::{CStr, CString},
    os::{
        fd::{AsRawFd, FromRawFd, RawFd},
        unix::fs::MetadataExt,
    },
};

pub const REPORT_FILE: &str = "mhi_validation_report.schema1.json";
pub const MANIFEST_FILE: &str = "validation_execution_manifest.schema1.json";
pub const SUMMARY_FILE: &str = "validation_summary.md";
const TABLES: [(&str, &str, &str); 6] = [
    (
        "cohort_coverage.csv",
        "cohort_coverage_csv",
        "endpoint_id,stratum_id,endpoint_kind,cohort_role,declared_count,eligible_count,excluded_count,not_applicable_count,exclusion_rate,exclusion_lower,exclusion_upper,evaluable_count,indeterminate_count,data_quality_insufficient_count,coverage,coverage_lower,coverage_upper,indeterminate_rate,indeterminate_lower,indeterminate_upper,data_quality_insufficient_rate,data_quality_insufficient_lower,data_quality_insufficient_upper,outcome",
    ),
    (
        "leakage_assessment.csv",
        "leakage_assessment_csv",
        "endpoint_id,stratum_id,record_id,separation_status,not_evaluated_reason,compared_development_record_ids,shared_artifact_ids,shared_source_sha256s,shared_experiment_ids,shared_family_ids,unknown_reasons,decision",
    ),
    (
        "mechanism_validation.csv",
        "mechanism_validation_csv",
        "endpoint_id,stratum_id,eligible_count,independent_family_count,support_count,critical_contradiction_count,declared_critical_falsification_count,not_assessed_or_other_count,support_fraction,support_lower,support_upper,contradiction_fraction,contradiction_lower,contradiction_upper,not_assessed_fraction,not_assessed_lower,not_assessed_upper,outcome",
    ),
    (
        "health_validation.csv",
        "health_validation_csv",
        "endpoint_id,stratum_id,eligible_count,independent_family_count,tp,tn,fp,fn,indeterminate,data_quality_insufficient,evaluable,coverage,coverage_lower,coverage_upper,indeterminate_rate,indeterminate_lower,indeterminate_upper,data_quality_insufficient_rate,data_quality_insufficient_lower,data_quality_insufficient_upper,sensitivity,sensitivity_lower,sensitivity_upper,specificity,specificity_lower,specificity_upper,false_positive_rate,false_positive_lower,false_positive_upper,false_negative_rate,false_negative_lower,false_negative_upper,balanced_accuracy,outcome",
    ),
    (
        "exclusion_ledger.csv",
        "exclusion_ledger_csv",
        "endpoint_id,stratum_id,record_id,primary_reason,secondary_reasons,assessed_source_key,reference_endpoint_id",
    ),
    (
        "compatibility_matrix.csv",
        "compatibility_matrix_csv",
        "record_id,source_role,relative_path,expected_kind,actual_kind,expected_schema,actual_schema,expected_file_sha256,actual_file_sha256,expected_artifact_id,actual_artifact_id,expected_semantic_sha256,actual_semantic_sha256,result",
    ),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedFileRecordV1 {
    relative_path: String,
    output_kind: String,
    byte_length: u64,
    sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationExecutionManifestV1 {
    schema_version: u32,
    output_kind: String,
    report_id: String,
    protocol_sha256: String,
    dataset_source: DatasetSourceReferenceV1,
    generated_files: Vec<GeneratedFileRecordV1>,
    publication_mode: String,
    software_version: String,
    git_commit: Option<String>,
}

/// The only production capability that can reach the raw filesystem publisher.
/// Its constructor is crate-private and replays the exact scientific authority
/// before any filesystem mutation is possible.
#[derive(Debug)]
pub(crate) struct AuthorizedMhiPublication<'a> {
    report: &'a MhiValidationReportV1,
}

pub(crate) fn authorize_publication<'a>(
    report: &'a MhiValidationReportV1,
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
) -> Result<AuthorizedMhiPublication<'a>, MhiValidationError> {
    report.validate_against(protocol, inputs)?;
    Ok(AuthorizedMhiPublication { report })
}

/// A private, filesystem-derived identity for a managed generation.  The
/// descriptor remains open for the lifetime of the proof, so later pathname
/// replacement cannot change the bytes being fingerprinted.
#[derive(Debug)]
struct BundleGeneration {
    descriptor: fs::File,
    device: u64,
    inode: u64,
    fingerprint: String,
}

#[cfg(unix)]
#[derive(Debug)]
struct DirectoryAuthority {
    descriptor: fs::File,
    display_path: PathBuf,
}

/// Publishes the fixed nine-file bundle. The staging tree is private to the
/// output parent and every generated byte is checked before it becomes visible.
pub(crate) fn publish_authorized_bundle(
    output_dir: &Path,
    authorization: &AuthorizedMhiPublication<'_>,
    overwrite: bool,
) -> Result<(), MhiValidationError> {
    #[cfg(not(unix))]
    {
        let _ = (output_dir, authorization, overwrite);
        return Err(MhiValidationError::UnsupportedAtomicPublicationFilesystem(
            output_dir.into(),
        ));
    }
    #[cfg(unix)]
    publish_authorized_bundle_unix(output_dir, authorization, overwrite)
}

#[cfg(unix)]
fn publish_authorized_bundle_unix(
    output_dir: &Path,
    authorization: &AuthorizedMhiPublication<'_>,
    overwrite: bool,
) -> Result<(), MhiValidationError> {
    let report = authorization.report;
    let parent = output_dir
        .parent()
        .ok_or_else(|| MhiValidationError::UnsafePath(output_dir.into()))?;
    let name = output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| valid_output_basename(name))
        .ok_or_else(|| MhiValidationError::UnsafePath(output_dir.into()))?;
    let parent_authority = open_parent_authority(parent)?;
    let output = parent_authority.display_path.join(name);
    let stage_name = format!(".{name}.phase-e-stage");
    let backup_name = format!(".{name}.phase-e-backup");
    let stage = parent_authority.display_path.join(&stage_name);
    let backup = parent_authority.display_path.join(&backup_name);
    publication_test_hook(PublicationTestPoint::AfterParentPinned, &stage, &output);
    let _publication_lock = acquire_publication_lock(&parent_authority, name)?;
    let stage_state =
        classify_publication_path_at(parent_authority.descriptor.as_raw_fd(), &stage_name, &stage)?;
    let backup_state = classify_publication_path_at(
        parent_authority.descriptor.as_raw_fd(),
        &backup_name,
        &backup,
    )?;
    if stage_state != PublicationPathState::Absent || backup_state != PublicationPathState::Absent {
        return Err(MhiValidationError::PublicationRecoveryResidue {
            output: output.clone(),
            output_state: classify_publication_path_at(
                parent_authority.descriptor.as_raw_fd(),
                name,
                &output,
            )?,
            stage_state,
            backup_state,
            remaining_paths: private_remaining_paths(&stage, &backup),
        });
    }
    let output_state =
        classify_publication_path_at(parent_authority.descriptor.as_raw_fd(), name, &output)?;
    if output_state == PublicationPathState::Symlink {
        return Err(MhiValidationError::UnsafePath(output));
    }
    let output_exists = output_state != PublicationPathState::Absent;
    if output_exists && !overwrite {
        return Err(MhiValidationError::OutputAlreadyExists(output));
    }
    let old_generation = if output_exists {
        if output_state != PublicationPathState::ValidManagedBundle {
            return Err(MhiValidationError::OutputNotManaged(output));
        }
        Some(managed_generation_at(
            parent_authority.descriptor.as_raw_fd(),
            name,
            &output,
        )?)
    } else {
        None
    };
    mkdir_at(&parent_authority.descriptor, &stage_name, &stage)?;
    if let Err(error) = sync_directory(&parent_authority.descriptor, &parent_authority.display_path)
    {
        return precommit_failure(&parent_authority, &stage_name, &stage, error);
    }
    let publication_mode = if output_exists {
        "replace_managed_bundle"
    } else {
        "create_new"
    };
    let staged = (|| -> Result<BundleGeneration, MhiValidationError> {
        let stage_descriptor =
            open_directory_at_fd(parent_authority.descriptor.as_raw_fd(), &stage_name, &stage)
                .map_err(|source| MhiValidationError::Io {
                    path: stage.clone(),
                    source,
                })?;
        let generated = write_stage(&stage_descriptor, &stage, report, publication_mode)?;
        publication_test_hook(
            PublicationTestPoint::BeforeStageVerification,
            &stage,
            &output,
        );
        verify_bundle_with_mode(&stage_descriptor, &stage, Some(publication_mode))?;
        publication_test_hook(
            PublicationTestPoint::AfterStageVerificationBeforeReread,
            &stage,
            &output,
        );
        let strict = read_artifact_at::<MhiValidationReportV1>(
            &stage_descriptor,
            REPORT_FILE,
            &stage.join(REPORT_FILE),
        )?;
        if strict.artifact != *report || generated.len() != 8 {
            return Err(MhiValidationError::Dataset(
                "published Phase-E stage does not match the evaluated report".into(),
            ));
        }
        sync_directory(&stage_descriptor, &stage)?;
        managed_generation_from_descriptor(stage_descriptor, &stage)
    })();
    let stage_generation = match staged {
        Ok(generation) => generation,
        Err(error) => {
            return precommit_failure(&parent_authority, &stage_name, &stage, error);
        }
    };

    if output_exists {
        publication_test_hook(PublicationTestPoint::BeforeManagedPrecheck, &stage, &output);
        let proof = compare_generation_at(
            parent_authority.descriptor.as_raw_fd(),
            name,
            &output,
            old_generation.as_ref().expect("managed output generation"),
        )?;
        if proof.identity_result != PublicationIdentityResult::Match
            || proof.fingerprint_result != PublicationFingerprintResult::Match
            || proof.state != PublicationPathState::ValidManagedBundle
        {
            let error = MhiValidationError::PublicationConcurrentManagedOutputChanged {
                output: output.clone(),
                output_state: proof.state,
                identity_result: proof.identity_result,
                fingerprint_result: proof.fingerprint_result,
                remaining_paths: vec![stage.clone()],
            };
            return precommit_failure(&parent_authority, &stage_name, &stage, error);
        }
        publication_test_hook(PublicationTestPoint::BeforeExchange, &stage, &output);
        if let Err(error) =
            atomic_exchange(&parent_authority.descriptor, &stage_name, name, &output)
        {
            return precommit_failure(&parent_authority, &stage_name, &stage, error);
        }
    } else {
        publication_test_hook(PublicationTestPoint::BeforeCreateCommit, &stage, &output);
        if let Err(error) =
            atomic_noreplace(&parent_authority.descriptor, &stage_name, name, &output)
        {
            return precommit_failure(&parent_authority, &stage_name, &stage, error);
        }
    }
    publication_test_hook(PublicationTestPoint::AfterExchange, &stage, &output);
    if let Err(error) = sync_directory(&parent_authority.descriptor, &parent_authority.display_path)
    {
        return Err(MhiValidationError::PublicationDurabilityUnconfirmed {
            output,
            operation: if output_exists {
                "replace_managed_bundle"
            } else {
                "create_new"
            },
            fsync_error: error.to_string(),
            remaining_paths: if output_exists {
                vec![stage.clone()]
            } else {
                Vec::new()
            },
        });
    }
    let visible_proof = compare_generation_at(
        parent_authority.descriptor.as_raw_fd(),
        name,
        &output,
        &stage_generation,
    )?;
    if visible_proof.identity_result != PublicationIdentityResult::Match
        || visible_proof.fingerprint_result != PublicationFingerprintResult::Match
        || visible_proof.state != PublicationPathState::ValidManagedBundle
    {
        return Err(
            MhiValidationError::PublicationCommittedVisibleOutputChanged {
                output,
                output_state: visible_proof.state,
                identity_result: visible_proof.identity_result,
                fingerprint_result: visible_proof.fingerprint_result,
                remaining_paths: if output_exists {
                    vec![stage.clone()]
                } else {
                    Vec::new()
                },
            },
        );
    }
    if output_exists {
        publication_test_hook(
            PublicationTestPoint::BeforeOldGenerationProof,
            &stage,
            &output,
        );
        let old_proof = compare_generation_at(
            parent_authority.descriptor.as_raw_fd(),
            &stage_name,
            &stage,
            old_generation.as_ref().expect("managed output generation"),
        )?;
        if old_proof.identity_result != PublicationIdentityResult::Match
            || old_proof.fingerprint_result != PublicationFingerprintResult::Match
            || old_proof.state != PublicationPathState::ValidManagedBundle
        {
            return Err(
                MhiValidationError::PublicationCommittedForeignSwapDetected {
                    output,
                    stage_state: old_proof.state,
                    identity_result: old_proof.identity_result,
                    fingerprint_result: old_proof.fingerprint_result,
                    remaining_paths: vec![stage.clone()],
                },
            );
        }
        // After an exchange the old managed generation is at `stage`. Move it
        // aside with a no-replace operation before deletion so any interrupted
        // cleanup leaves explicit operator-visible residue rather than an
        // ambiguous output namespace.
        publication_test_hook(
            PublicationTestPoint::BeforeOldGenerationCleanup,
            &stage,
            &output,
        );
        let cleanup = (|| -> Result<(), MhiValidationError> {
            atomic_noreplace(
                &parent_authority.descriptor,
                &stage_name,
                &backup_name,
                &backup,
            )?;
            sync_directory(&parent_authority.descriptor, &parent_authority.display_path)?;
            remove_tree_reverse_at(&parent_authority.descriptor, &backup_name, &backup)?;
            sync_directory(&parent_authority.descriptor, &parent_authority.display_path)?;
            Ok(())
        })();
        if let Err(cleanup_error) = cleanup {
            return Err(MhiValidationError::PublicationCommittedCleanupFailed {
                output,
                stage_state: classify_publication_path_at(
                    parent_authority.descriptor.as_raw_fd(),
                    &stage_name,
                    &stage,
                )?,
                backup_state: classify_publication_path_at(
                    parent_authority.descriptor.as_raw_fd(),
                    &backup_name,
                    &backup,
                )?,
                remaining_paths: private_remaining_paths(&stage, &backup),
                cleanup_error: cleanup_error.to_string(),
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct GenerationProof {
    state: PublicationPathState,
    identity_result: PublicationIdentityResult,
    fingerprint_result: PublicationFingerprintResult,
}

fn precommit_failure(
    parent: &DirectoryAuthority,
    stage_name: &str,
    stage: &Path,
    primary_error: MhiValidationError,
) -> Result<(), MhiValidationError> {
    match cleanup_stage_at(&parent.descriptor, stage_name, stage) {
        Ok(()) => Err(primary_error),
        Err(_cleanup_error) => Err(MhiValidationError::PublicationStagingCleanupFailed {
            primary_error: primary_error.to_string(),
            remaining_paths: remaining_paths(stage),
        }),
    }
}

#[cfg(unix)]
fn classify_publication_path_at(
    parent: RawFd,
    name: &str,
    display_path: &Path,
) -> Result<PublicationPathState, MhiValidationError> {
    let descriptor = match open_child_nofollow(parent, name, display_path) {
        Ok(descriptor) => descriptor,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PublicationPathState::Absent);
        }
        Err(error) if is_symlink_error(&error) => return Ok(PublicationPathState::Symlink),
        Err(source) => {
            return Err(MhiValidationError::Io {
                path: display_path.into(),
                source,
            });
        }
    };
    let metadata = descriptor
        .metadata()
        .map_err(|source| MhiValidationError::Io {
            path: display_path.into(),
            source,
        })?;
    if !metadata.is_dir() {
        return Ok(PublicationPathState::Unmanaged);
    }
    if verify_bundle_with_mode(&descriptor, display_path, None).is_ok() {
        Ok(PublicationPathState::ValidManagedBundle)
    } else {
        Ok(PublicationPathState::Unmanaged)
    }
}

#[cfg(unix)]
fn compare_generation_at(
    parent: RawFd,
    name: &str,
    display_path: &Path,
    expected: &BundleGeneration,
) -> Result<GenerationProof, MhiValidationError> {
    let held_metadata =
        expected
            .descriptor
            .metadata()
            .map_err(|source| MhiValidationError::Io {
                path: display_path.into(),
                source,
            })?;
    if held_metadata.dev() != expected.device || held_metadata.ino() != expected.inode {
        return Ok(GenerationProof {
            state: PublicationPathState::Unmanaged,
            identity_result: PublicationIdentityResult::Mismatch,
            fingerprint_result: PublicationFingerprintResult::NotEvaluated,
        });
    }
    let descriptor = match open_directory_at_fd(parent, name, display_path) {
        Ok(descriptor) => descriptor,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(GenerationProof {
                state: PublicationPathState::Absent,
                identity_result: PublicationIdentityResult::Unavailable,
                fingerprint_result: PublicationFingerprintResult::NotEvaluated,
            });
        }
        Err(error) if is_symlink_error(&error) => {
            return Ok(GenerationProof {
                state: PublicationPathState::Symlink,
                identity_result: PublicationIdentityResult::Unavailable,
                fingerprint_result: PublicationFingerprintResult::NotEvaluated,
            });
        }
        Err(source) => {
            return Err(MhiValidationError::Io {
                path: display_path.into(),
                source,
            });
        }
    };
    let metadata = descriptor
        .metadata()
        .map_err(|source| MhiValidationError::Io {
            path: display_path.into(),
            source,
        })?;
    let identity_matches = metadata.dev() == expected.device && metadata.ino() == expected.inode;
    if !identity_matches {
        return Ok(GenerationProof {
            state: if verify_bundle_with_mode(&descriptor, display_path, None).is_ok() {
                PublicationPathState::ValidManagedBundle
            } else {
                PublicationPathState::Unmanaged
            },
            identity_result: PublicationIdentityResult::Mismatch,
            fingerprint_result: PublicationFingerprintResult::NotEvaluated,
        });
    }
    let fingerprint = match bundle_fingerprint_from_descriptor(&descriptor, display_path) {
        Ok(fingerprint) => fingerprint,
        Err(_) => {
            return Ok(GenerationProof {
                state: if verify_bundle_with_mode(&descriptor, display_path, None).is_ok() {
                    PublicationPathState::ValidManagedBundle
                } else {
                    PublicationPathState::Unmanaged
                },
                identity_result: PublicationIdentityResult::Match,
                fingerprint_result: PublicationFingerprintResult::NotEvaluated,
            });
        }
    };
    if fingerprint != expected.fingerprint {
        return Ok(GenerationProof {
            state: if verify_bundle_with_mode(&descriptor, display_path, None).is_ok() {
                PublicationPathState::ValidManagedBundle
            } else {
                PublicationPathState::Unmanaged
            },
            identity_result: PublicationIdentityResult::Match,
            fingerprint_result: PublicationFingerprintResult::Mismatch,
        });
    }
    Ok(GenerationProof {
        state: if verify_bundle_with_mode(&descriptor, display_path, None).is_ok() {
            PublicationPathState::ValidManagedBundle
        } else {
            PublicationPathState::Unmanaged
        },
        identity_result: PublicationIdentityResult::Match,
        fingerprint_result: PublicationFingerprintResult::Match,
    })
}

fn private_remaining_paths(stage: &Path, backup: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.extend(remaining_paths(stage));
    paths.extend(remaining_paths(backup));
    paths.sort();
    paths
}

fn remaining_paths(path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if !path_exists_or_symlink_unchecked(path) {
        return paths;
    }
    paths.push(path.to_path_buf());
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.file_type().is_dir()
        && !metadata.file_type().is_symlink()
        && let Ok(entries) = fs::read_dir(path)
    {
        for entry in entries.flatten() {
            paths.extend(remaining_paths(&entry.path()));
        }
    }
    paths.sort();
    paths
}

fn path_exists_or_symlink_unchecked(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Test-only probes model a noncooperating writer at exactly the committed
/// generation boundaries.  Production builds compile these calls to no-ops;
/// the filesystem state machine itself remains the authority under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicationTestPoint {
    AfterParentPinned,
    BeforeManagedPrecheck,
    BeforeCreateCommit,
    BeforeExchange,
    AfterExchange,
    BeforeStageVerification,
    AfterStageVerificationBeforeReread,
    BeforeOldGenerationProof,
    BeforeOldGenerationCleanup,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicationTestHook {
    ReplaceOutputBeforePrecheck,
    CreateOutputBeforeCommit,
    ReplaceOutputBeforeExchange,
    MutateOutputBeforeExchange,
    ReplaceVisibleOutputAfterExchange,
    MutateVisibleOutputAfterExchange,
    MutateOldStageBeforeProof,
    PrecreateBackupBeforeCleanup,
    MutateStagedChecksum,
    MutateStagedReportBeforeReread,
    AddManifestSelfRecord,
    WrongCreateManifestMode,
    WrongReplaceManifestMode,
    AddManifestTimestamp,
    AddManifestUnknownField,
    AddExtraGeneratedFile,
    RemoveManagedFile,
    ReplaceManagedFileWithSymlink,
    ReplacePinnedParentPath,
}

#[cfg(test)]
static PUBLICATION_TEST_HOOK: std::sync::Mutex<Option<PublicationTestHook>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
fn set_publication_test_hook(hook: Option<PublicationTestHook>) {
    *PUBLICATION_TEST_HOOK
        .lock()
        .expect("publication test hook lock") = hook;
}

#[cfg(test)]
fn publication_test_hook(point: PublicationTestPoint, _stage: &Path, output: &Path) {
    let hook = *PUBLICATION_TEST_HOOK
        .lock()
        .expect("publication test hook lock");
    let Some(hook) = hook else {
        return;
    };
    let matches_point = matches!(
        (hook, point),
        (
            PublicationTestHook::CreateOutputBeforeCommit,
            PublicationTestPoint::BeforeCreateCommit
        ) | (
            PublicationTestHook::ReplaceOutputBeforePrecheck,
            PublicationTestPoint::BeforeManagedPrecheck
        ) | (
            PublicationTestHook::ReplaceOutputBeforeExchange,
            PublicationTestPoint::BeforeExchange
        ) | (
            PublicationTestHook::MutateOutputBeforeExchange,
            PublicationTestPoint::BeforeExchange
        ) | (
            PublicationTestHook::ReplaceVisibleOutputAfterExchange,
            PublicationTestPoint::AfterExchange
        ) | (
            PublicationTestHook::MutateVisibleOutputAfterExchange,
            PublicationTestPoint::AfterExchange
        ) | (
            PublicationTestHook::PrecreateBackupBeforeCleanup,
            PublicationTestPoint::BeforeOldGenerationCleanup
        ) | (
            PublicationTestHook::MutateOldStageBeforeProof,
            PublicationTestPoint::BeforeOldGenerationProof
        ) | (
            PublicationTestHook::MutateStagedChecksum,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::AddManifestSelfRecord,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::WrongCreateManifestMode,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::WrongReplaceManifestMode,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::AddManifestTimestamp,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::AddManifestUnknownField,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::AddExtraGeneratedFile,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::RemoveManagedFile,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::ReplaceManagedFileWithSymlink,
            PublicationTestPoint::BeforeStageVerification
        ) | (
            PublicationTestHook::MutateStagedReportBeforeReread,
            PublicationTestPoint::AfterStageVerificationBeforeReread
        ) | (
            PublicationTestHook::ReplacePinnedParentPath,
            PublicationTestPoint::AfterParentPinned
        )
    );
    if !matches_point {
        return;
    }
    *PUBLICATION_TEST_HOOK
        .lock()
        .expect("publication test hook lock") = None;
    match hook {
        PublicationTestHook::CreateOutputBeforeCommit => {
            fs::create_dir(output).expect("create competing output");
            fs::write(output.join("sentinel.txt"), b"concurrent competitor")
                .expect("write competing output");
        }
        PublicationTestHook::ReplaceOutputBeforePrecheck
        | PublicationTestHook::ReplaceOutputBeforeExchange
        | PublicationTestHook::ReplaceVisibleOutputAfterExchange => {
            let competitor = output.with_file_name(format!(
                ".{}.phase-e-foreign-competitor",
                output
                    .file_name()
                    .expect("output file name")
                    .to_string_lossy()
            ));
            fs::rename(output, &competitor).expect("move generation to competitor");
            fs::create_dir(output).expect("create foreign output");
            fs::write(output.join("sentinel.txt"), b"foreign competitor")
                .expect("write foreign output");
        }
        PublicationTestHook::MutateOutputBeforeExchange
        | PublicationTestHook::MutateVisibleOutputAfterExchange => {
            fs::write(output.join(REPORT_FILE), b"mutated generation")
                .expect("mutate held output generation");
        }
        PublicationTestHook::MutateOldStageBeforeProof => {
            fs::write(_stage.join(REPORT_FILE), b"mutated old generation")
                .expect("mutate old stage");
        }
        PublicationTestHook::PrecreateBackupBeforeCleanup => {
            let name = output
                .file_name()
                .expect("output file name")
                .to_string_lossy();
            fs::create_dir(output.with_file_name(format!(".{name}.phase-e-backup")))
                .expect("precreate backup residue");
        }
        PublicationTestHook::MutateStagedChecksum => {
            let path = _stage.join(SUMMARY_FILE);
            fs::write(path, b"checksum mutation").expect("mutate staged checksum");
        }
        PublicationTestHook::MutateStagedReportBeforeReread => {
            fs::write(_stage.join(REPORT_FILE), b"not an artifact").expect("mutate staged report");
        }
        PublicationTestHook::AddManifestSelfRecord => {
            mutate_manifest(_stage, |manifest| {
                let mut record = manifest["generated_files"][0].clone();
                record["relative_path"] = Value::String(MANIFEST_FILE.into());
                manifest["generated_files"]
                    .as_array_mut()
                    .expect("generated files")
                    .push(record);
            });
        }
        PublicationTestHook::WrongCreateManifestMode => {
            mutate_manifest(_stage, |manifest| {
                manifest["publication_mode"] = Value::String("replace_managed_bundle".into());
            });
        }
        PublicationTestHook::WrongReplaceManifestMode => {
            mutate_manifest(_stage, |manifest| {
                manifest["publication_mode"] = Value::String("create_new".into());
            });
        }
        PublicationTestHook::AddManifestTimestamp => {
            mutate_manifest(_stage, |manifest| {
                manifest["timestamp"] = Value::String("2026-08-22T00:00:00Z".into());
            });
        }
        PublicationTestHook::AddManifestUnknownField => {
            mutate_manifest(_stage, |manifest| {
                manifest["extra"] = Value::String("forbidden".into());
            });
        }
        PublicationTestHook::AddExtraGeneratedFile => {
            fs::write(_stage.join("unexpected.txt"), b"unexpected").expect("extra generated file");
        }
        PublicationTestHook::RemoveManagedFile => {
            fs::remove_file(_stage.join(SUMMARY_FILE)).expect("remove managed file");
        }
        PublicationTestHook::ReplacePinnedParentPath => {
            let parent = output.parent().expect("output parent");
            let foreign = parent.with_file_name(format!(
                ".{}.phase-e-foreign-parent",
                parent.file_name().expect("parent name").to_string_lossy()
            ));
            fs::rename(parent, &foreign).expect("move pinned parent namespace");
            fs::create_dir(parent).expect("replace parent namespace");
        }
        PublicationTestHook::ReplaceManagedFileWithSymlink => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let victim = _stage.join(REPORT_FILE);
                let foreign = _stage.with_file_name(".phase-e-foreign-managed-report");
                fs::rename(&victim, &foreign).expect("move managed file to foreign namespace");
                symlink(&foreign, &victim).expect("replace managed file with symlink");
            }
        }
    }
}

#[cfg(test)]
fn mutate_manifest(stage: &Path, mutate: impl FnOnce(&mut Value)) {
    let path = stage.join(MANIFEST_FILE);
    let mut value: Value =
        serde_json::from_slice(&fs::read(&path).expect("manifest")).expect("JSON");
    mutate(&mut value);
    fs::write(
        path,
        serde_json::to_vec_pretty(&value).expect("manifest JSON"),
    )
    .expect("mutated manifest");
}

#[cfg(not(test))]
fn publication_test_hook(_point: PublicationTestPoint, _stage: &Path, _output: &Path) {}

#[cfg(unix)]
fn write_stage(
    stage: &fs::File,
    display_stage: &Path,
    report: &MhiValidationReportV1,
    publication_mode: &str,
) -> Result<Vec<GeneratedFileRecordV1>, MhiValidationError> {
    let report_path = display_stage.join(REPORT_FILE);
    test_fault(PublicationFaultOperation::Write, &report_path)?;
    let report_bytes = serialize_artifact(&report_path, report)?;
    let report_bytes = normalize_json_bytes(&report_path, &report_bytes)?;
    write_bytes_at(stage, REPORT_FILE, &report_path, &report_bytes)?;
    write_bytes_at(
        stage,
        SUMMARY_FILE,
        &display_stage.join(SUMMARY_FILE),
        summary_markdown(report).as_bytes(),
    )?;

    let tables_path = display_stage.join("tables");
    mkdir_at(stage, "tables", &tables_path)?;
    let tables =
        open_directory_at_fd(stage.as_raw_fd(), "tables", &tables_path).map_err(|source| {
            MhiValidationError::Io {
                path: tables_path.clone(),
                source,
            }
        })?;
    for (name, _, header) in TABLES {
        write_bytes_at(
            &tables,
            name,
            &tables_path.join(name),
            &table_bytes(name, header, report)?,
        )?;
    }
    sync_directory(&tables, &tables_path)?;
    let generated_files = generated_file_records(stage, display_stage)?;
    let manifest = ValidationExecutionManifestV1 {
        schema_version: 1,
        output_kind: "mhi_validation_execution_manifest".into(),
        report_id: report.report_id.clone(),
        protocol_sha256: report.protocol.source_file_sha256.clone(),
        dataset_source: report.dataset.source.clone(),
        generated_files: generated_files.clone(),
        publication_mode: publication_mode.into(),
        software_version: report.provenance.software_version.clone(),
        git_commit: report.provenance.git_commit.clone(),
    };
    write_json_value_at(
        stage,
        MANIFEST_FILE,
        &display_stage.join(MANIFEST_FILE),
        serde_json::to_value(manifest)?,
    )?;
    sync_directory(stage, display_stage)?;
    Ok(generated_files)
}

fn table_bytes(
    name: &str,
    header: &str,
    report: &MhiValidationReportV1,
) -> Result<Vec<u8>, MhiValidationError> {
    let rows = table_rows(name, report)?;
    csv_document(header, rows)
}

fn table_rows(
    name: &str,
    report: &MhiValidationReportV1,
) -> Result<Vec<Vec<String>>, MhiValidationError> {
    match name {
        "cohort_coverage.csv" => Ok(report
            .cohorts
            .iter()
            .map(|row| {
                let mut cells = vec![
                    row.endpoint_id.clone(),
                    row.stratum_id.clone(),
                    token(&row.endpoint_kind),
                    token(&row.cohort_role),
                    row.declared_count.to_string(),
                    row.eligible_count.to_string(),
                    row.excluded_count.to_string(),
                    row.not_applicable_count.to_string(),
                ];
                cells.extend(metric_cells(Some(&row.exclusion_rate)));
                cells.push(optional_count(row.evaluable_count));
                cells.push(optional_count(row.indeterminate_count));
                cells.push(optional_count(row.data_quality_insufficient_count));
                cells.extend(metric_cells(row.coverage.as_ref()));
                cells.extend(metric_cells(row.indeterminate_rate.as_ref()));
                cells.extend(metric_cells(row.data_quality_insufficient_rate.as_ref()));
                cells.push(token(&row.outcome));
                cells
            })
            .collect()),
        "leakage_assessment.csv" => Ok(report
            .leakage_assessment
            .iter()
            .map(|row| {
                vec![
                    row.endpoint_id.clone(),
                    row.stratum_id.clone(),
                    row.record_id.clone(),
                    optional_token(row.separation_status.as_ref()),
                    optional_token(row.not_evaluated_reason.as_ref()),
                    json_cell(&row.compared_development_record_ids),
                    json_cell(&row.shared_artifact_ids),
                    json_cell(&row.shared_source_sha256s),
                    json_cell(&row.shared_experiment_ids),
                    json_cell(&row.shared_family_ids),
                    json_cell(&row.unknown_reasons),
                    token(&row.decision),
                ]
            })
            .collect()),
        "mechanism_validation.csv" => Ok(report
            .mechanism_results
            .iter()
            .map(|row| {
                let mut cells = vec![
                    row.endpoint_id.clone(),
                    row.stratum_id.clone(),
                    row.eligible_count.to_string(),
                    row.independent_family_count.to_string(),
                    row.support_count.to_string(),
                    row.critical_contradiction_count.to_string(),
                    row.declared_critical_falsification_count.to_string(),
                    row.not_assessed_or_other_count.to_string(),
                ];
                cells.extend(metric_cells(Some(&row.support_fraction)));
                cells.extend(metric_cells(Some(&row.contradiction_fraction)));
                cells.extend(metric_cells(Some(&row.not_assessed_fraction)));
                cells.push(token(&row.outcome));
                cells
            })
            .collect()),
        "health_validation.csv" => Ok(report
            .health_results
            .iter()
            .map(|row| {
                let mut cells = vec![
                    row.endpoint_id.clone(),
                    row.stratum_id.clone(),
                    row.eligible_count.to_string(),
                    row.independent_family_count.to_string(),
                    row.tp.to_string(),
                    row.tn.to_string(),
                    row.fp.to_string(),
                    row.r#fn.to_string(),
                    row.indeterminate.to_string(),
                    row.data_quality_insufficient.to_string(),
                    row.evaluable.to_string(),
                ];
                for metric in [
                    &row.coverage,
                    &row.indeterminate_rate,
                    &row.data_quality_insufficient_rate,
                    &row.sensitivity,
                    &row.specificity,
                    &row.false_positive_rate,
                    &row.false_negative_rate,
                ] {
                    cells.extend(metric_cells(Some(metric)));
                }
                cells.push(match &row.balanced_accuracy {
                    crate::results::BalancedAccuracyV1::Available { point_estimate, .. } => {
                        float_token(*point_estimate)
                    }
                    crate::results::BalancedAccuracyV1::Unavailable { .. } => "NA".into(),
                });
                cells.push(token(&row.outcome));
                cells
            })
            .collect()),
        "exclusion_ledger.csv" => Ok(report
            .exclusions
            .iter()
            .map(|row| {
                vec![
                    row.endpoint_id.clone(),
                    row.stratum_id.clone(),
                    row.record_id.clone(),
                    token(&row.primary_reason),
                    json_cell(&row.secondary_reasons),
                    row.assessed_source_key
                        .as_ref()
                        .map_or_else(|| "NA".into(), json_cell),
                    row.reference_endpoint_id
                        .clone()
                        .unwrap_or_else(|| "NA".into()),
                ]
            })
            .collect()),
        "compatibility_matrix.csv" => Ok(report
            .compatibility
            .iter()
            .map(|row| {
                vec![
                    row.record_id.clone().unwrap_or_else(|| "NA".into()),
                    token(&row.source_role),
                    row.relative_path.clone(),
                    row.expected_kind
                        .map_or_else(|| "NA".into(), |value| value.to_string()),
                    row.actual_kind
                        .map_or_else(|| "NA".into(), |value| value.to_string()),
                    row.expected_schema.to_string(),
                    row.actual_schema.to_string(),
                    row.expected_file_sha256.clone(),
                    row.actual_file_sha256.clone(),
                    row.expected_artifact_id
                        .as_ref()
                        .map_or_else(|| "NA".into(), |value| value.0.clone()),
                    row.actual_artifact_id
                        .as_ref()
                        .map_or_else(|| "NA".into(), |value| value.0.clone()),
                    row.expected_semantic_sha256
                        .clone()
                        .unwrap_or_else(|| "NA".into()),
                    row.actual_semantic_sha256
                        .clone()
                        .unwrap_or_else(|| "NA".into()),
                    token(&row.result),
                ]
            })
            .collect()),
        _ => Err(MhiValidationError::Dataset("unknown Phase-E table".into())),
    }
}

fn csv_document(header: &str, rows: Vec<Vec<String>>) -> Result<Vec<u8>, MhiValidationError> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(Vec::new());
    writer
        .write_record(header.split(','))
        .map_err(|error| MhiValidationError::Dataset(format!("CSV header: {error}")))?;
    for row in rows {
        writer
            .write_record(row)
            .map_err(|error| MhiValidationError::Dataset(format!("CSV row: {error}")))?;
    }
    writer
        .into_inner()
        .map_err(|error| MhiValidationError::Dataset(format!("CSV finalization: {error}")))
}

#[cfg(unix)]
fn generated_file_records(
    stage: &fs::File,
    display_stage: &Path,
) -> Result<Vec<GeneratedFileRecordV1>, MhiValidationError> {
    let mut entries = vec![
        (REPORT_FILE.to_string(), "mhi_validation_report"),
        (SUMMARY_FILE.to_string(), "validation_summary_markdown"),
    ];
    for (name, kind, _) in TABLES {
        entries.push((format!("tables/{name}"), kind));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
        .into_iter()
        .map(|(relative_path, output_kind)| {
            let bytes = read_relative_bundle_file(stage, &relative_path, display_stage)?;
            Ok(GeneratedFileRecordV1 {
                relative_path,
                output_kind: output_kind.into(),
                byte_length: bytes.len() as u64,
                sha256: sha256(&bytes),
            })
        })
        .collect()
}

#[cfg(unix)]
fn verify_bundle_with_mode(
    descriptor: &fs::File,
    display_path: &Path,
    expected_publication_mode: Option<&str>,
) -> Result<(), MhiValidationError> {
    let expected_root = BTreeSet::from([
        REPORT_FILE.to_string(),
        MANIFEST_FILE.to_string(),
        SUMMARY_FILE.to_string(),
        "tables".into(),
    ]);
    let actual_root = read_child_names(descriptor, display_path)?;
    if actual_root != expected_root {
        return Err(MhiValidationError::Dataset(
            "managed bundle has an unexpected root entry".into(),
        ));
    }
    let tables_path = display_path.join("tables");
    let tables =
        open_directory_at_fd(descriptor.as_raw_fd(), "tables", &tables_path).map_err(|source| {
            MhiValidationError::Io {
                path: tables_path.clone(),
                source,
            }
        })?;
    let expected_tables = TABLES
        .iter()
        .map(|(name, _, _)| (*name).to_string())
        .collect::<BTreeSet<_>>();
    if read_child_names(&tables, &tables_path)? != expected_tables {
        return Err(MhiValidationError::Dataset(
            "managed bundle has an unexpected table entry".into(),
        ));
    }
    let mut fixed_files = generated_paths();
    fixed_files.push(MANIFEST_FILE.into());
    for relative in fixed_files {
        let child = display_path.join(&relative);
        let _ = open_relative_regular_file(descriptor, &relative, &child)?;
    }
    let manifest: ValidationExecutionManifestV1 = serde_json::from_slice(
        &read_relative_bundle_file(descriptor, MANIFEST_FILE, display_path)?,
    )?;
    if manifest.schema_version != 1
        || manifest.output_kind != "mhi_validation_execution_manifest"
        || !matches!(
            manifest.publication_mode.as_str(),
            "create_new" | "replace_managed_bundle"
        )
        || manifest.generated_files.len() != 8
    {
        return Err(MhiValidationError::Dataset(
            "managed bundle manifest violates the Phase-E schema".into(),
        ));
    }
    if expected_publication_mode.is_some_and(|expected| manifest.publication_mode != expected) {
        return Err(MhiValidationError::Dataset(
            "managed bundle publication mode does not match the requested operation".into(),
        ));
    }
    let expected = generated_paths();
    let actual = manifest
        .generated_files
        .iter()
        .map(|record| record.relative_path.clone())
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(MhiValidationError::Dataset(
            "managed bundle files do not match the fixed Phase-E set".into(),
        ));
    }
    for record in &manifest.generated_files {
        let bytes = read_relative_bundle_file(descriptor, &record.relative_path, display_path)?;
        if record.byte_length != bytes.len() as u64 || record.sha256 != sha256(&bytes) {
            return Err(MhiValidationError::Dataset(
                "managed bundle checksum does not match its manifest".into(),
            ));
        }
    }
    let strict = read_artifact_at::<MhiValidationReportV1>(
        descriptor,
        REPORT_FILE,
        &display_path.join(REPORT_FILE),
    )?;
    if strict.artifact.report_id != manifest.report_id
        || strict.artifact.protocol.source_file_sha256 != manifest.protocol_sha256
        || strict.artifact.dataset.source != manifest.dataset_source
        || strict.artifact.provenance.software_version != manifest.software_version
        || strict.artifact.provenance.git_commit != manifest.git_commit
    {
        return Err(MhiValidationError::Dataset(
            "managed bundle manifest and report identities disagree".into(),
        ));
    }
    Ok(())
}

fn generated_paths() -> Vec<String> {
    let mut paths = vec![REPORT_FILE.into(), SUMMARY_FILE.into()];
    paths.extend(TABLES.iter().map(|(name, _, _)| format!("tables/{name}")));
    paths.sort();
    paths
}

#[cfg(unix)]
fn managed_generation_at(
    parent: RawFd,
    name: &str,
    display_path: &Path,
) -> Result<BundleGeneration, MhiValidationError> {
    managed_generation_from_descriptor(
        open_directory_at_fd(parent, name, display_path).map_err(|source| {
            if is_symlink_error(&source) {
                MhiValidationError::UnsafePath(display_path.into())
            } else {
                MhiValidationError::Io {
                    path: display_path.into(),
                    source,
                }
            }
        })?,
        display_path,
    )
}

#[cfg(unix)]
fn managed_generation_from_descriptor(
    descriptor: fs::File,
    display_path: &Path,
) -> Result<BundleGeneration, MhiValidationError> {
    let metadata = descriptor
        .metadata()
        .map_err(|source| MhiValidationError::Io {
            path: display_path.into(),
            source,
        })?;
    if !metadata.is_dir() {
        return Err(MhiValidationError::UnsafePath(display_path.into()));
    }
    verify_bundle_with_mode(&descriptor, display_path, None)?;
    let fingerprint = bundle_fingerprint_from_descriptor(&descriptor, display_path)?;
    Ok(BundleGeneration {
        descriptor,
        device: metadata.dev(),
        inode: metadata.ino(),
        fingerprint,
    })
}

#[cfg(unix)]
fn bundle_fingerprint_from_descriptor(
    descriptor: &fs::File,
    display_path: &Path,
) -> Result<String, MhiValidationError> {
    let mut hasher = Sha256::new();
    hasher.update(b"mhi_managed_bundle_fingerprint_v1\0");
    let mut paths = generated_paths();
    paths.push(MANIFEST_FILE.into());
    paths.sort();
    for relative in paths {
        let bytes = read_relative_bundle_file(descriptor, &relative, display_path)?;
        hasher.update((relative.len() as u64).to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update((bytes.len() as u64).to_be_bytes());
        hasher.update(Sha256::digest(&bytes));
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sorted_json(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(sorted_json).collect()),
        Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, sorted_json(value)))
                    .collect(),
            )
        }
        other => other,
    }
}

fn summary_markdown(report: &MhiValidationReportV1) -> String {
    let mut output = String::from("# MHI Validation Summary\n\n");
    output.push_str("## Identity\n\n");
    let approval = report.approval.as_ref();
    output.push_str(&markdown_table(
        &["key", "value"],
        vec![
            vec!["report_id".into(), report.report_id.clone()],
            vec!["protocol_id".into(), report.protocol.protocol_id.clone()],
            vec![
                "protocol_sha256".into(),
                report.protocol.source_file_sha256.clone(),
            ],
            vec!["dataset_id".into(), report.dataset.dataset_id.clone()],
            vec![
                "dataset_source_file_sha256".into(),
                dataset_source_file_sha256(report).into(),
            ],
            vec![
                "approval_record_id".into(),
                approval
                    .map(|value| value.approval_record_id.clone())
                    .unwrap_or_else(|| "NA".into()),
            ],
            vec![
                "approval_trust_store_sha256".into(),
                approval
                    .map(|value| value.trust_store_sha256.clone())
                    .unwrap_or_else(|| "NA".into()),
            ],
            vec![
                "software_version".into(),
                report.provenance.software_version.clone(),
            ],
            vec![
                "git_commit".into(),
                report
                    .provenance
                    .git_commit
                    .clone()
                    .unwrap_or_else(|| "NA".into()),
            ],
        ],
    ));
    for (heading, table, header) in [
        ("Cohort Coverage", "cohort_coverage.csv", TABLES[0].2),
        ("Leakage", "leakage_assessment.csv", TABLES[1].2),
        (
            "Mechanism Endpoints",
            "mechanism_validation.csv",
            TABLES[2].2,
        ),
        ("Health Endpoints", "health_validation.csv", TABLES[3].2),
        ("Exclusions", "exclusion_ledger.csv", TABLES[4].2),
    ] {
        output.push_str(&format!("## {heading}\n\n"));
        output.push_str(&markdown_table(
            &header.split(',').collect::<Vec<_>>(),
            table_rows(table, report).expect("fixed output table"),
        ));
    }
    output.push_str("## Release Claims\n\n");
    output.push_str(&markdown_table(
        &[
            "claim_id",
            "requested_level",
            "statement",
            "domain",
            "supporting_endpoint_ids",
            "approval_record_id",
            "outcome",
        ],
        report
            .release_claims
            .iter()
            .map(|claim| {
                vec![
                    claim.claim_id.clone(),
                    token(&claim.requested_level),
                    claim.statement.clone(),
                    json_cell(&claim.domain),
                    json_cell(&claim.supporting_endpoint_ids),
                    approval
                        .map(|value| value.approval_record_id.clone())
                        .unwrap_or_else(|| "NA".into()),
                    token(&claim.outcome),
                ]
            })
            .collect(),
    ));
    output.push_str("## Overall Status\n\n");
    output.push_str(&format!("outcome: {}\n\n", token(&report.overall_status)));
    output.push_str("## Limitations\n\n");
    let limitations = limitations(report);
    if limitations.is_empty() {
        output.push_str("- NONE\n");
    } else {
        for limitation in limitations {
            output.push_str("- ");
            output.push_str(&markdown_escape(&limitation));
            output.push('\n');
        }
    }
    output
}

fn markdown_table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let mut output = String::new();
    output.push_str("| ");
    output.push_str(&headers.join(" | "));
    output.push_str(" |\n| ");
    output.push_str(&vec!["---"; headers.len()].join(" | "));
    output.push_str(" |\n");
    for row in rows {
        output.push_str("| ");
        output.push_str(
            &row.iter()
                .map(|cell| markdown_escape(cell))
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n");
    }
    output.push('\n');
    output
}

fn limitations(report: &MhiValidationReportV1) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    if let Some(approval) = &report.approval {
        values.extend(
            approval
                .limitations
                .iter()
                .map(|value| format!("approval:{value}")),
        );
    }
    for result in &report.mechanism_results {
        values.extend(result.limitations.iter().map(|value| {
            format!(
                "endpoint:{}:{}:{value}",
                result.endpoint_id, result.stratum_id
            )
        }));
    }
    for result in &report.health_results {
        values.extend(result.limitations.iter().map(|value| {
            format!(
                "endpoint:{}:{}:{value}",
                result.endpoint_id, result.stratum_id
            )
        }));
    }
    values.extend(report.warnings.iter().map(|warning| {
        format!(
            "warning:{}:{}:{}",
            token(&warning.code),
            warning.related_id,
            warning.detail
        )
    }));
    values
}

fn markdown_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "<br>")
}

fn metric_cells(metric: Option<&MetricValueV1>) -> Vec<String> {
    match metric {
        Some(MetricValueV1::Available {
            point_estimate,
            lower_confidence_bound,
            upper_confidence_bound,
            ..
        }) => vec![
            float_token(*point_estimate),
            float_token(*lower_confidence_bound),
            float_token(*upper_confidence_bound),
        ],
        Some(MetricValueV1::Unavailable { .. }) | None => vec!["NA".into(); 3],
    }
}

fn optional_count(value: Option<u64>) -> String {
    value.map_or_else(|| "NA".into(), |value| value.to_string())
}

fn token<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .expect("closed enum serialization")
        .as_str()
        .expect("closed enum must serialize as a token")
        .to_string()
}

fn optional_token<T: Serialize>(value: Option<&T>) -> String {
    value.map_or_else(|| "NA".into(), token)
}

fn json_cell<T: Serialize>(value: &T) -> String {
    String::from_utf8(serde_jcs::to_vec(value).expect("typed output is JCS serializable"))
        .expect("JCS is UTF-8")
}

fn float_token(value: f64) -> String {
    serde_json::to_string(&if value == 0.0 { 0.0 } else { value }).expect("finite Phase-E output")
}

fn dataset_source_file_sha256(report: &MhiValidationReportV1) -> &str {
    match &report.dataset.source {
        DatasetSourceReferenceV1::Known {
            source_file_sha256, ..
        }
        | DatasetSourceReferenceV1::LegacyUnknown {
            source_file_sha256, ..
        } => source_file_sha256,
    }
}

fn valid_output_basename(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', '\0'])
}

/// The lock file deliberately persists.  Its identity is part of the
/// publication namespace, and unlinking/recreating it would allow two writers
/// to hold locks on different files.
#[cfg(unix)]
fn acquire_publication_lock(
    parent: &DirectoryAuthority,
    output_name: &str,
) -> Result<fs::File, MhiValidationError> {
    let lock_name = format!(".{output_name}.phase-e-publish.lock");
    let lock = parent.display_path.join(&lock_name);
    let (file, created) = match open_at_raw(
        parent.descriptor.as_raw_fd(),
        &lock_name,
        O_RDWR | O_CLOEXEC | O_NOFOLLOW,
        0,
    ) {
        Ok(descriptor) => (unsafe { fs::File::from_raw_fd(descriptor) }, false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let descriptor = open_at_raw(
                parent.descriptor.as_raw_fd(),
                &lock_name,
                O_RDWR | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW,
                0o600,
            )
            .map_err(|source| {
                if is_symlink_error(&source) {
                    MhiValidationError::UnsafePath(lock.clone())
                } else {
                    MhiValidationError::Io {
                        path: lock.clone(),
                        source,
                    }
                }
            })?;
            (unsafe { fs::File::from_raw_fd(descriptor) }, true)
        }
        Err(error) if is_symlink_error(&error) => {
            return Err(MhiValidationError::UnsafePath(lock));
        }
        Err(source) => {
            return Err(MhiValidationError::Io { path: lock, source });
        }
    };
    let metadata = file.metadata().map_err(|source| MhiValidationError::Io {
        path: lock.clone(),
        source,
    })?;
    #[cfg(unix)]
    let regular_single_link = metadata.is_file() && metadata.nlink() == 1;
    #[cfg(not(unix))]
    let regular_single_link = metadata.is_file();
    if !regular_single_link || metadata.len() != 0 {
        return Err(MhiValidationError::PublicationLockFileInvalid(lock));
    }
    lock_exclusive_nonblocking(&file).map_err(|source| {
        if source.kind() == std::io::ErrorKind::WouldBlock {
            MhiValidationError::PublicationLocked(lock.clone())
        } else {
            MhiValidationError::Io {
                path: lock.clone(),
                source,
            }
        }
    })?;
    if metadata.nlink() == 1 && metadata.len() == 0 {
        // A newly created lock is already mode 0600. The descriptor-relative
        // chmod keeps this invariant even when a pre-existing file had a
        // broader mode.
        let _ = unsafe { native_fchmod(file.as_raw_fd(), 0o600) };
    }
    if created {
        test_fault(PublicationFaultOperation::SyncFile, &lock)?;
        file.sync_all().map_err(|source| MhiValidationError::Io {
            path: lock.clone(),
            source,
        })?;
        sync_directory(&parent.descriptor, &parent.display_path)?;
    }
    Ok(file)
}

#[cfg(unix)]
fn lock_exclusive_nonblocking(file: &fs::File) -> std::io::Result<()> {
    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    let result = unsafe { native_flock(std::os::fd::AsRawFd::as_raw_fd(file), LOCK_EX | LOCK_NB) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(not(unix))]
fn lock_exclusive_nonblocking(_file: &fs::File) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Phase-E publication locking requires Unix advisory locks",
    ))
}

#[derive(Debug, Clone, Copy)]
enum RenameOperation {
    NoReplace,
    Exchange,
}

#[cfg(unix)]
fn atomic_noreplace(
    parent: &fs::File,
    source_name: &str,
    destination_name: &str,
    destination: &Path,
) -> Result<(), MhiValidationError> {
    atomic_rename_at(
        parent,
        source_name,
        destination_name,
        destination,
        RenameOperation::NoReplace,
    )
}

#[cfg(unix)]
fn atomic_exchange(
    parent: &fs::File,
    source_name: &str,
    destination_name: &str,
    destination: &Path,
) -> Result<(), MhiValidationError> {
    atomic_rename_at(
        parent,
        source_name,
        destination_name,
        destination,
        RenameOperation::Exchange,
    )
}

#[cfg(unix)]
fn atomic_rename_at(
    parent: &fs::File,
    source_name: &str,
    destination_name: &str,
    destination: &Path,
    operation: RenameOperation,
) -> Result<(), MhiValidationError> {
    if let Some(result) = test_rename_fault(operation, destination) {
        return result;
    }
    let from = CString::new(source_name.as_bytes())
        .map_err(|_| MhiValidationError::UnsafePath(destination.into()))?;
    let to = CString::new(destination_name.as_bytes())
        .map_err(|_| MhiValidationError::UnsafePath(destination.into()))?;
    let flags = match operation {
        RenameOperation::NoReplace => native_rename_noreplace_flag(),
        RenameOperation::Exchange => native_rename_exchange_flag(),
    };
    let result = unsafe {
        native_rename_at(
            parent.as_raw_fd(),
            from.as_ptr(),
            parent.as_raw_fd(),
            to.as_ptr(),
            flags,
        )
    };
    if result == 0 {
        return Ok(());
    }
    let source_error = std::io::Error::last_os_error();
    if matches!(operation, RenameOperation::NoReplace)
        && source_error.kind() == std::io::ErrorKind::AlreadyExists
    {
        return Err(
            MhiValidationError::PublicationConcurrentDestinationCreated {
                output: destination.into(),
            },
        );
    }
    if matches!(
        source_error.raw_os_error(),
        Some(38) | Some(45) | Some(78) | Some(95)
    ) {
        return Err(MhiValidationError::UnsupportedAtomicPublicationFilesystem(
            destination.into(),
        ));
    }
    Err(MhiValidationError::Io {
        path: destination.into(),
        source: source_error,
    })
}

#[cfg(target_os = "macos")]
const fn native_rename_noreplace_flag() -> u32 {
    0x0000_0004 // RENAME_EXCL
}
#[cfg(target_os = "macos")]
const fn native_rename_exchange_flag() -> u32 {
    0x0000_0002 // RENAME_SWAP
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
const fn native_rename_noreplace_flag() -> u32 {
    0x0000_0001 // RENAME_NOREPLACE
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
const fn native_rename_exchange_flag() -> u32 {
    0x0000_0002 // RENAME_EXCHANGE
}
#[cfg(target_os = "macos")]
unsafe fn native_flock(fd: i32, operation: i32) -> i32 {
    unsafe { flock(fd, operation) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_flock(fd: i32, operation: i32) -> i32 {
    unsafe { flock(fd, operation) }
}

#[cfg(unix)]
fn sync_directory(descriptor: &fs::File, display_path: &Path) -> Result<(), MhiValidationError> {
    test_fault(PublicationFaultOperation::SyncDirectory, display_path)?;
    descriptor
        .sync_all()
        .map_err(|source| MhiValidationError::Io {
            path: display_path.into(),
            source,
        })
}

#[cfg(unix)]
fn cleanup_stage_at(
    parent: &fs::File,
    stage_name: &str,
    display_stage: &Path,
) -> Result<(), MhiValidationError> {
    match open_child_nofollow(parent.as_raw_fd(), stage_name, display_stage) {
        Ok(_) => remove_tree_reverse_at(parent, stage_name, display_stage),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) if is_symlink_error(&error) => unlink_at(parent, stage_name, 0, display_stage),
        Err(source) => Err(MhiValidationError::Io {
            path: display_stage.into(),
            source,
        }),
    }
}

#[cfg(unix)]
fn remove_tree_reverse_at(
    parent: &fs::File,
    name: &str,
    display_path: &Path,
) -> Result<(), MhiValidationError> {
    let child = match open_child_nofollow(parent.as_raw_fd(), name, display_path) {
        Ok(child) => child,
        Err(error) if is_symlink_error(&error) => {
            return unlink_at(parent, name, 0, display_path);
        }
        Err(source) => {
            return Err(MhiValidationError::Io {
                path: display_path.into(),
                source,
            });
        }
    };
    let metadata = child.metadata().map_err(|source| MhiValidationError::Io {
        path: display_path.into(),
        source,
    })?;
    if !metadata.is_dir() {
        return unlink_at(parent, name, 0, display_path);
    }
    let child_names = read_child_names(&child, display_path)?;
    for child_name in child_names {
        let child_path = display_path.join(&child_name);
        let child_open = open_child_nofollow(child.as_raw_fd(), &child_name, &child_path);
        if let Ok(child_descriptor) = child_open {
            let child_metadata =
                child_descriptor
                    .metadata()
                    .map_err(|source| MhiValidationError::Io {
                        path: child_path.clone(),
                        source,
                    })?;
            drop(child_descriptor);
            if child_metadata.is_dir() {
                remove_tree_reverse_at(&child, &child_name, &child_path)?;
            } else {
                unlink_at(&child, &child_name, 0, &child_path)?;
            }
        } else if let Err(error) = child_open {
            if is_symlink_error(&error) {
                unlink_at(&child, &child_name, 0, &child_path)?;
            } else {
                return Err(MhiValidationError::Io {
                    path: child_path,
                    source: error,
                });
            }
        }
    }
    unlink_at(parent, name, AT_REMOVEDIR, display_path)
}

#[cfg(unix)]
fn open_parent_authority(path: &Path) -> Result<DirectoryAuthority, MhiValidationError> {
    let mut current: Option<fs::File> = None;
    let components = path.components().collect::<Vec<_>>();
    for (index, component) in components.into_iter().enumerate() {
        match component {
            std::path::Component::RootDir => {
                current = Some(open_directory_at_fd(AT_FDCWD, "/", path).map_err(|source| {
                    MhiValidationError::Io {
                        path: path.into(),
                        source,
                    }
                })?);
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(component) => {
                let name = component
                    .to_str()
                    .ok_or_else(|| MhiValidationError::UnsafePath(path.into()))?;
                let parent_fd = current.as_ref().map_or(AT_FDCWD, AsRawFd::as_raw_fd);
                current = Some(match open_directory_at_fd(parent_fd, name, path) {
                    Ok(descriptor) => descriptor,
                    Err(source)
                        if index + 1 < path.components().count() && is_symlink_error(&source) =>
                    {
                        open_directory_at_follow(parent_fd, name, path).map_err(|follow_error| {
                            MhiValidationError::Io {
                                path: path.into(),
                                source: follow_error,
                            }
                        })?
                    }
                    Err(source) if is_symlink_error(&source) => {
                        return Err(MhiValidationError::UnsafePath(path.into()));
                    }
                    Err(source) => {
                        return Err(MhiValidationError::Io {
                            path: path.into(),
                            source,
                        });
                    }
                });
            }
            std::path::Component::ParentDir | std::path::Component::Prefix(_) => {
                return Err(MhiValidationError::UnsafePath(path.into()));
            }
        }
    }
    let descriptor =
        current.unwrap_or(open_directory_at_fd(AT_FDCWD, ".", path).map_err(|source| {
            MhiValidationError::Io {
                path: path.into(),
                source,
            }
        })?);
    let metadata = descriptor
        .metadata()
        .map_err(|source| MhiValidationError::Io {
            path: path.into(),
            source,
        })?;
    if !metadata.is_dir() {
        return Err(MhiValidationError::UnsafePath(path.into()));
    }
    Ok(DirectoryAuthority {
        descriptor,
        display_path: path.into(),
    })
}

#[cfg(unix)]
fn open_directory_at_fd(
    parent: RawFd,
    name: &str,
    _display_path: &Path,
) -> std::io::Result<fs::File> {
    let descriptor = match open_at_raw(
        parent,
        name,
        O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW,
        0,
    ) {
        Ok(descriptor) => descriptor,
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => {
            let descriptor = open_at_raw(parent, name, O_RDONLY | O_CLOEXEC | O_NOFOLLOW, 0)?;
            drop(unsafe { fs::File::from_raw_fd(descriptor) });
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    Ok(unsafe { fs::File::from_raw_fd(descriptor) })
}

#[cfg(unix)]
fn open_directory_at_follow(
    parent: RawFd,
    name: &str,
    display_path: &Path,
) -> std::io::Result<fs::File> {
    let descriptor = open_at_raw(parent, name, O_RDONLY | O_DIRECTORY | O_CLOEXEC, 0)?;
    let file = unsafe { fs::File::from_raw_fd(descriptor) };
    if !file.metadata()?.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            display_path.display().to_string(),
        ));
    }
    Ok(file)
}

#[cfg(unix)]
fn open_child_nofollow(
    parent: RawFd,
    name: &str,
    display_path: &Path,
) -> std::io::Result<fs::File> {
    let _ = display_path;
    let descriptor = open_at_raw(parent, name, O_RDONLY | O_CLOEXEC | O_NOFOLLOW, 0)?;
    Ok(unsafe { fs::File::from_raw_fd(descriptor) })
}

#[cfg(unix)]
fn open_relative_regular_file(
    generation: &fs::File,
    relative: &str,
    display_path: &Path,
) -> Result<fs::File, MhiValidationError> {
    let file = if let Some((directory_name, file_name)) = relative.split_once('/') {
        let directory_path = display_path.join(directory_name);
        let directory =
            open_directory_at_fd(generation.as_raw_fd(), directory_name, &directory_path).map_err(
                |source| MhiValidationError::Io {
                    path: directory_path,
                    source,
                },
            )?;
        open_child_nofollow(
            directory.as_raw_fd(),
            file_name,
            &display_path.join(relative),
        )
    } else {
        open_child_nofollow(
            generation.as_raw_fd(),
            relative,
            &display_path.join(relative),
        )
    };
    let file_path = display_path.join(relative);
    let file = file.map_err(|source| {
        if is_symlink_error(&source) {
            MhiValidationError::UnsafePath(file_path.clone())
        } else {
            MhiValidationError::Io {
                path: file_path.clone(),
                source,
            }
        }
    })?;
    let metadata = file.metadata().map_err(|source| MhiValidationError::Io {
        path: file_path.clone(),
        source,
    })?;
    if !metadata.is_file() {
        return Err(MhiValidationError::UnsafePath(file_path));
    }
    Ok(file)
}

#[cfg(unix)]
fn read_relative_bundle_file(
    generation: &fs::File,
    relative: &str,
    display_path: &Path,
) -> Result<Vec<u8>, MhiValidationError> {
    let mut file = open_relative_regular_file(generation, relative, display_path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|source| MhiValidationError::Io {
            path: display_path.join(relative),
            source,
        })?;
    Ok(bytes)
}

#[cfg(unix)]
fn read_artifact_at<T: crate::domain::VersionedArtifact>(
    generation: &fs::File,
    name: &str,
    display_path: &Path,
) -> Result<crate::domain::StrictArtifactRead<T>, MhiValidationError> {
    let bytes = read_relative_bundle_file(generation, name, display_path)?;
    read_artifact_strict_bytes(&display_path.join(name), &bytes).map_err(MhiValidationError::from)
}

#[cfg(unix)]
fn normalize_json_bytes(path: &Path, bytes: &[u8]) -> Result<Vec<u8>, MhiValidationError> {
    let value: Value = serde_json::from_slice(bytes)?;
    let mut normalized = serde_json::to_string_pretty(&sorted_json(value))?.into_bytes();
    normalized.push(b'\n');
    let _ = path;
    Ok(normalized)
}

#[cfg(unix)]
fn write_bytes_at(
    directory: &fs::File,
    name: &str,
    display_path: &Path,
    bytes: &[u8],
) -> Result<(), MhiValidationError> {
    test_fault(PublicationFaultOperation::Write, display_path)?;
    let descriptor = open_at_raw(
        directory.as_raw_fd(),
        name,
        O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC | O_NOFOLLOW,
        0o600,
    )
    .map_err(|source| MhiValidationError::Io {
        path: display_path.into(),
        source,
    })?;
    let mut file = unsafe { fs::File::from_raw_fd(descriptor) };
    file.write_all(bytes)
        .map_err(|source| MhiValidationError::Io {
            path: display_path.into(),
            source,
        })?;
    test_fault(PublicationFaultOperation::SyncFile, display_path)?;
    file.sync_all().map_err(|source| MhiValidationError::Io {
        path: display_path.into(),
        source,
    })
}

#[cfg(unix)]
fn write_json_value_at(
    directory: &fs::File,
    name: &str,
    display_path: &Path,
    value: Value,
) -> Result<(), MhiValidationError> {
    let mut bytes = serde_json::to_string_pretty(&sorted_json(value))?.into_bytes();
    bytes.push(b'\n');
    write_bytes_at(directory, name, display_path, &bytes)
}

#[cfg(unix)]
fn read_child_names(
    directory: &fs::File,
    display_path: &Path,
) -> Result<BTreeSet<String>, MhiValidationError> {
    let duplicate = open_at_raw(
        directory.as_raw_fd(),
        ".",
        O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW,
        0,
    )
    .map_err(|source| MhiValidationError::Io {
        path: display_path.into(),
        source,
    })?;
    let stream = unsafe { native_fdopendir(duplicate) };
    if stream.is_null() {
        drop(unsafe { fs::File::from_raw_fd(duplicate) });
        return Err(MhiValidationError::Io {
            path: display_path.into(),
            source: std::io::Error::last_os_error(),
        });
    }
    let mut names = BTreeSet::new();
    loop {
        clear_errno();
        let entry = next_dir_entry(stream);
        if entry.is_null() {
            let errno = current_errno();
            if errno != 0 {
                // The stream must be closed on the error path, but this
                // cleanup failure cannot change the original enumeration
                // error or the exact-set proof that was not completed.
                unsafe { native_closedir(stream) };
                return Err(MhiValidationError::Io {
                    path: display_path.into(),
                    source: std::io::Error::from_raw_os_error(errno),
                });
            }
            break;
        }
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_str()
            .map_err(|_| MhiValidationError::Dataset("non-UTF-8 output path".into()))?;
        if name != "." && name != ".." {
            names.insert(name.to_owned());
        }
    }
    // `readdir` has returned normal EOF, so the exact names are complete.
    // `closedir` only releases the stream resource at this point; R2 has no
    // cleanup-success requirement whose failure could alter that proof.
    unsafe { native_closedir(stream) };
    Ok(names)
}

#[cfg(unix)]
fn mkdir_at(
    directory: &fs::File,
    name: &str,
    display_path: &Path,
) -> Result<(), MhiValidationError> {
    let path = CString::new(name.as_bytes())
        .map_err(|_| MhiValidationError::UnsafePath(display_path.into()))?;
    let result = unsafe { native_mkdirat(directory.as_raw_fd(), path.as_ptr(), 0o700) };
    if result == 0 {
        Ok(())
    } else {
        Err(MhiValidationError::Io {
            path: display_path.into(),
            source: std::io::Error::last_os_error(),
        })
    }
}

#[cfg(unix)]
fn unlink_at(
    directory: &fs::File,
    name: &str,
    flags: i32,
    display_path: &Path,
) -> Result<(), MhiValidationError> {
    test_fault(PublicationFaultOperation::Delete, display_path)?;
    let path = CString::new(name.as_bytes())
        .map_err(|_| MhiValidationError::UnsafePath(display_path.into()))?;
    let result = unsafe { native_unlinkat(directory.as_raw_fd(), path.as_ptr(), flags) };
    if result != 0 {
        return Err(MhiValidationError::Io {
            path: display_path.into(),
            source: std::io::Error::last_os_error(),
        });
    }
    sync_directory(directory, display_path.parent().unwrap_or(display_path))
}

#[cfg(unix)]
fn open_at_raw(parent: RawFd, name: &str, flags: i32, mode: i32) -> std::io::Result<RawFd> {
    let path = CString::new(name.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "NUL in path"))?;
    let descriptor = unsafe { native_openat(parent, path.as_ptr(), flags, mode) };
    if descriptor < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(descriptor)
    }
}

#[cfg(unix)]
fn is_symlink_error(error: &std::io::Error) -> bool {
    matches!(error.raw_os_error(), Some(40) | Some(62))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
const O_RDONLY: i32 = 0;
#[cfg(target_os = "macos")]
const O_WRONLY: i32 = 0x0001;
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this marker as applying to the paired macOS constants.
// Do not treat these Linux variants as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
const O_WRONLY: i32 = 0x0001;
#[cfg(target_os = "macos")]
const O_RDWR: i32 = 0x0002;
#[cfg(target_os = "linux")]
const O_RDWR: i32 = 0x0002;
#[cfg(target_os = "macos")]
const O_CREAT: i32 = 0x0200;
#[cfg(target_os = "linux")]
const O_CREAT: i32 = 0x0040;
#[cfg(target_os = "macos")]
const O_EXCL: i32 = 0x0800;
#[cfg(target_os = "linux")]
const O_EXCL: i32 = 0x0080;
#[cfg(target_os = "macos")]
const O_TRUNC: i32 = 0x0400;
#[cfg(target_os = "linux")]
const O_TRUNC: i32 = 0x0200;
#[cfg(target_os = "macos")]
const O_CLOEXEC: i32 = 0x0100_0000;
#[cfg(target_os = "linux")]
const O_CLOEXEC: i32 = 0x0008_0000;
#[cfg(target_os = "macos")]
const O_DIRECTORY: i32 = 0x0010_0000;
#[cfg(target_os = "linux")]
const O_DIRECTORY: i32 = 0x0001_0000;
#[cfg(target_os = "macos")]
const O_NOFOLLOW: i32 = 0x0000_0100;
#[cfg(target_os = "linux")]
const O_NOFOLLOW: i32 = 0x0002_0000;
#[cfg(target_os = "macos")]
const AT_FDCWD: i32 = -2;
#[cfg(target_os = "linux")]
const AT_FDCWD: i32 = -100;
#[cfg(target_os = "macos")]
const AT_REMOVEDIR: i32 = 0x0080;
#[cfg(target_os = "linux")]
const AT_REMOVEDIR: i32 = 0x0200;

#[cfg(target_os = "macos")]
#[repr(C)]
struct NativeDirent {
    d_ino: u64,
    d_seekoff: u64,
    d_reclen: u16,
    d_namlen: u16,
    d_type: u8,
    d_name: [std::ffi::c_char; 1024],
}

// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
#[repr(C)]
struct NativeDirent {
    d_ino: u64,
    d_off: i64,
    d_reclen: u16,
    d_type: u8,
    d_name: [std::ffi::c_char; 256],
}

#[repr(C)]
struct NativeDir {
    _private: [u8; 0],
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn __error() -> *mut i32;
    fn openat(fd: i32, path: *const std::ffi::c_char, flags: i32, ...) -> i32;
    fn mkdirat(fd: i32, path: *const std::ffi::c_char, mode: u16) -> i32;
    fn unlinkat(fd: i32, path: *const std::ffi::c_char, flags: i32) -> i32;
    #[link_name = "renameatx_np"]
    fn renameatx_np(
        fromfd: i32,
        from: *const std::ffi::c_char,
        tofd: i32,
        to: *const std::ffi::c_char,
        flags: u32,
    ) -> i32;
    fn flock(fd: i32, operation: i32) -> i32;
    fn fchmod(fd: i32, mode: u16) -> i32;
    fn fdopendir(fd: i32) -> *mut NativeDir;
    fn readdir(dirp: *mut NativeDir) -> *mut NativeDirent;
    fn closedir(dirp: *mut NativeDir) -> i32;
}

// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn __errno_location() -> *mut i32;
    fn openat(fd: i32, path: *const std::ffi::c_char, flags: i32, ...) -> i32;
    fn mkdirat(fd: i32, path: *const std::ffi::c_char, mode: u32) -> i32;
    fn unlinkat(fd: i32, path: *const std::ffi::c_char, flags: i32) -> i32;
    fn flock(fd: i32, operation: i32) -> i32;
    fn fchmod(fd: i32, mode: u32) -> i32;
    fn fdopendir(fd: i32) -> *mut NativeDir;
    fn readdir(dirp: *mut NativeDir) -> *mut NativeDirent;
    fn closedir(dirp: *mut NativeDir) -> i32;
    fn syscall(number: std::ffi::c_long, ...) -> std::ffi::c_long;
}

#[cfg(target_os = "macos")]
unsafe fn native_openat(fd: i32, path: *const std::ffi::c_char, flags: i32, mode: i32) -> i32 {
    unsafe { openat(fd, path, flags, mode) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_openat(fd: i32, path: *const std::ffi::c_char, flags: i32, mode: i32) -> i32 {
    unsafe { openat(fd, path, flags, mode) }
}
#[cfg(target_os = "macos")]
unsafe fn native_mkdirat(fd: i32, path: *const std::ffi::c_char, mode: i32) -> i32 {
    unsafe { mkdirat(fd, path, mode as u16) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_mkdirat(fd: i32, path: *const std::ffi::c_char, mode: i32) -> i32 {
    unsafe { mkdirat(fd, path, mode as u32) }
}
#[cfg(target_os = "macos")]
unsafe fn native_unlinkat(fd: i32, path: *const std::ffi::c_char, flags: i32) -> i32 {
    unsafe { unlinkat(fd, path, flags) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_unlinkat(fd: i32, path: *const std::ffi::c_char, flags: i32) -> i32 {
    unsafe { unlinkat(fd, path, flags) }
}
#[cfg(target_os = "macos")]
unsafe fn native_rename_at(
    fromfd: i32,
    from: *const std::ffi::c_char,
    tofd: i32,
    to: *const std::ffi::c_char,
    flags: u32,
) -> i32 {
    unsafe { renameatx_np(fromfd, from, tofd, to, flags) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_rename_at(
    fromfd: i32,
    from: *const std::ffi::c_char,
    tofd: i32,
    to: *const std::ffi::c_char,
    flags: u32,
) -> i32 {
    #[cfg(target_arch = "x86_64")]
    const RENAMEAT2_SYSCALL: std::ffi::c_long = 316;
    #[cfg(target_arch = "aarch64")]
    const RENAMEAT2_SYSCALL: std::ffi::c_long = 276;
    #[cfg(target_arch = "arm")]
    const RENAMEAT2_SYSCALL: std::ffi::c_long = 382;
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
    const RENAMEAT2_SYSCALL: std::ffi::c_long = -1;
    if RENAMEAT2_SYSCALL < 0 {
        return -1;
    }
    unsafe { syscall(RENAMEAT2_SYSCALL, fromfd, from, tofd, to, flags) as i32 }
}
#[cfg(target_os = "macos")]
unsafe fn native_fchmod(fd: i32, mode: i32) -> i32 {
    unsafe { fchmod(fd, mode as u16) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_fchmod(fd: i32, mode: i32) -> i32 {
    unsafe { fchmod(fd, mode as u32) }
}
#[cfg(target_os = "macos")]
unsafe fn native_fdopendir(fd: i32) -> *mut NativeDir {
    unsafe { fdopendir(fd) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_fdopendir(fd: i32) -> *mut NativeDir {
    unsafe { fdopendir(fd) }
}
#[cfg(target_os = "macos")]
unsafe fn native_readdir(dir: *mut NativeDir) -> *mut NativeDirent {
    unsafe { readdir(dir) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_readdir(dir: *mut NativeDir) -> *mut NativeDirent {
    unsafe { readdir(dir) }
}
#[cfg(target_os = "macos")]
unsafe fn native_closedir(dir: *mut NativeDir) -> i32 {
    unsafe { closedir(dir) }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
unsafe fn native_closedir(dir: *mut NativeDir) -> i32 {
    unsafe { closedir(dir) }
}

#[cfg(target_os = "macos")]
fn native_errno_location() -> *mut i32 {
    // SAFETY: macOS `__error` returns the address of the calling thread's
    // errno slot, which remains valid for the duration of this access.
    unsafe { __error() }
}
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat this path as part of the MHI V1 supported platform surface.
#[cfg(target_os = "linux")]
fn native_errno_location() -> *mut i32 {
    // SAFETY: Linux `__errno_location` returns the address of the calling
    // thread's errno slot, which remains valid for the duration of this access.
    unsafe { __errno_location() }
}

#[cfg(unix)]
fn clear_errno() {
    // SAFETY: `native_errno_location` identifies this thread's errno slot;
    // writing zero immediately before `readdir` establishes the required
    // clean EOF/error distinction without affecting another thread.
    unsafe { *native_errno_location() = 0 };
}

#[cfg(unix)]
fn current_errno() -> i32 {
    // SAFETY: `native_errno_location` identifies this thread's errno slot;
    // it is read only after `readdir` returned NULL, when errno is its error
    // indicator. Successful `readdir` results never consult errno.
    unsafe { *native_errno_location() }
}

#[cfg(test)]
fn set_errno_for_test(errno: i32) {
    // SAFETY: the test writes only the current thread's errno slot so the
    // stale-errno and injected-readdir cases model the native contract.
    unsafe { *native_errno_location() = errno };
}

#[cfg(not(test))]
#[cfg(unix)]
fn next_dir_entry(stream: *mut NativeDir) -> *mut NativeDirent {
    // SAFETY: `stream` is the non-null stream returned by `fdopendir`, and
    // ownership remains with this function's caller until `closedir`.
    unsafe { native_readdir(stream) }
}

#[cfg(test)]
fn next_dir_entry(stream: *mut NativeDir) -> *mut NativeDirent {
    if test_readdir_fault() {
        set_errno_for_test(TEST_READDIR_ERRNO);
        return std::ptr::null_mut();
    }
    // SAFETY: `stream` is the non-null stream returned by `fdopendir`, and
    // ownership remains with this function's caller until `closedir`.
    unsafe { native_readdir(stream) }
}

fn sha256(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

#[derive(Debug, Clone, Copy)]
enum PublicationFaultOperation {
    Write,
    SyncFile,
    SyncDirectory,
    Delete,
}

#[cfg(test)]
// MHI_V1_DEFERRED_LINUX_SUPPORT:
// Retained for possible future Linux support. Linux is not a supported,
// release-validated, or approval-gating platform for MHI V1 Phase E under R3.
// Do not treat the Linux portion of this historical evidence as part of the
// MHI V1 supported platform surface.
// EIO is 5 on both exact historical Phase-E targets: macOS and Linux.
const TEST_READDIR_ERRNO: i32 = 5;

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
enum PublicationFault {
    WriteAt(usize),
    SyncFileAt(usize),
    SyncDirectoryAt(usize),
    NoReplaceUnsupported,
    NoReplaceFailure,
    ExchangeUnsupported,
    ExchangeFailure,
    DeleteAt(usize),
    ReadDirAt(usize),
}

#[cfg(test)]
#[derive(Debug, Default)]
struct PublicationFaultState {
    fault: Option<PublicationFault>,
    writes: usize,
    sync_files: usize,
    sync_directories: usize,
    deletes: usize,
    readdir_calls: usize,
}

#[cfg(test)]
static PUBLICATION_FAULT: std::sync::Mutex<PublicationFaultState> =
    std::sync::Mutex::new(PublicationFaultState {
        fault: None,
        writes: 0,
        sync_files: 0,
        sync_directories: 0,
        deletes: 0,
        readdir_calls: 0,
    });

#[cfg(test)]
fn set_publication_fault(fault: Option<PublicationFault>) {
    let mut state = PUBLICATION_FAULT.lock().expect("publication fault lock");
    state.fault = fault;
    state.writes = 0;
    state.sync_files = 0;
    state.sync_directories = 0;
    state.deletes = 0;
    state.readdir_calls = 0;
}

#[cfg(test)]
fn test_readdir_fault() -> bool {
    let mut state = PUBLICATION_FAULT.lock().expect("publication fault lock");
    state.readdir_calls += 1;
    let matches = matches!(
        state.fault,
        Some(PublicationFault::ReadDirAt(expected)) if state.readdir_calls == expected
    );
    if matches {
        state.fault = None;
    }
    matches
}

#[cfg(test)]
fn publication_read_dir_calls() -> usize {
    PUBLICATION_FAULT
        .lock()
        .expect("publication fault lock")
        .readdir_calls
}

fn test_fault(operation: PublicationFaultOperation, path: &Path) -> Result<(), MhiValidationError> {
    #[cfg(test)]
    {
        let mut state = PUBLICATION_FAULT.lock().expect("publication fault lock");
        let ordinal = match operation {
            PublicationFaultOperation::Write => {
                state.writes += 1;
                state.writes
            }
            PublicationFaultOperation::SyncFile => {
                state.sync_files += 1;
                state.sync_files
            }
            PublicationFaultOperation::SyncDirectory => {
                state.sync_directories += 1;
                state.sync_directories
            }
            PublicationFaultOperation::Delete => {
                state.deletes += 1;
                state.deletes
            }
        };
        let matches = match (state.fault, operation) {
            (Some(PublicationFault::WriteAt(expected)), PublicationFaultOperation::Write) => {
                ordinal == expected
            }
            (Some(PublicationFault::SyncFileAt(expected)), PublicationFaultOperation::SyncFile) => {
                ordinal == expected
            }
            (
                Some(PublicationFault::SyncDirectoryAt(expected)),
                PublicationFaultOperation::SyncDirectory,
            ) => ordinal == expected,
            (Some(PublicationFault::DeleteAt(expected)), PublicationFaultOperation::Delete) => {
                ordinal == expected
            }
            _ => false,
        };
        if matches {
            state.fault = None;
            return Err(MhiValidationError::Io {
                path: path.into(),
                source: std::io::Error::other("injected Phase-E publication filesystem failure"),
            });
        }
    }
    let _ = (operation, path);
    Ok(())
}

#[cfg(test)]
fn test_rename_fault(
    operation: RenameOperation,
    destination: &Path,
) -> Option<Result<(), MhiValidationError>> {
    let mut state = PUBLICATION_FAULT.lock().expect("publication fault lock");
    let fault = match (operation, state.fault) {
        (RenameOperation::NoReplace, Some(PublicationFault::NoReplaceUnsupported)) => Some(Err(
            MhiValidationError::UnsupportedAtomicPublicationFilesystem(destination.into()),
        )),
        (RenameOperation::NoReplace, Some(PublicationFault::NoReplaceFailure)) => {
            Some(Err(MhiValidationError::Io {
                path: destination.into(),
                source: std::io::Error::other("injected Phase-E no-replace failure"),
            }))
        }
        (RenameOperation::Exchange, Some(PublicationFault::ExchangeUnsupported)) => Some(Err(
            MhiValidationError::UnsupportedAtomicPublicationFilesystem(destination.into()),
        )),
        (RenameOperation::Exchange, Some(PublicationFault::ExchangeFailure)) => {
            Some(Err(MhiValidationError::Io {
                path: destination.into(),
                source: std::io::Error::other("injected Phase-E exchange failure"),
            }))
        }
        _ => None,
    };
    if fault.is_some() {
        state.fault = None;
    }
    fault
}

#[cfg(not(test))]
fn test_rename_fault(
    _operation: RenameOperation,
    _destination: &Path,
) -> Option<Result<(), MhiValidationError>> {
    None
}

#[allow(dead_code)]
fn sibling_private_path(output: &Path, suffix: &str) -> Option<PathBuf> {
    output
        .file_name()
        .map(|name| output.with_file_name(format!(".{}.{}", name.to_string_lossy(), suffix)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mhi_validation::{
        MhiValidationProtocolV1, ValidationInputs, evaluate_mhi_validation,
    };
    use std::{
        path::Path,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    static PUBLICATION_TEST_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn temporary_parent(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "phase_e_publication_{label}_{}_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("publication parent");
        root
    }

    fn software_fixture_authority() -> (MhiValidationProtocolV1, ValidationInputs) {
        let root = temporary_parent("report_input");
        let dataset = root.join("dataset/input.schema1.json");
        let lineage = root.join("dataset/lineage/complete.schema1.json");
        fs::create_dir_all(lineage.parent().expect("lineage parent")).expect("input layout");
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase_e");
        fs::copy(
            fixture_root.join("dataset/software_valid.schema1.json"),
            &dataset,
        )
        .expect("dataset fixture");
        fs::copy(fixture_root.join("lineage/complete.schema1.json"), &lineage)
            .expect("lineage fixture");
        let sources = root.join("dataset/sources");
        fs::create_dir_all(&sources).expect("source input layout");
        for (fixture_name, staged_name) in [
            (
                "mechanism/supported.schema4.json",
                "mechanism_a.schema4.json",
            ),
            (
                "mechanism/all_levels.schema4.json",
                "mechanism_c.schema4.json",
            ),
            (
                "health/within_baseline.schema4.json",
                "health_a.schema4.json",
            ),
            ("health/alert.schema4.json", "health_c.schema4.json"),
        ] {
            fs::copy(fixture_root.join(fixture_name), sources.join(staged_name))
                .expect("scientific source fixture");
        }
        let protocol_bytes =
            fs::read(fixture_root.join("protocol/software_valid.toml")).expect("protocol fixture");
        let protocol = MhiValidationProtocolV1::from_toml(
            std::str::from_utf8(&protocol_bytes).expect("UTF-8"),
        )
        .expect("protocol");
        let protocol_sha256 = MhiValidationProtocolV1::sha256_of_bytes(&protocol_bytes);
        let inputs =
            ValidationInputs::read(&protocol, &protocol_sha256, &dataset).expect("fixture inputs");
        fs::remove_dir_all(root).expect("input cleanup");
        (protocol, inputs)
    }

    fn software_fixture_report() -> crate::results::MhiValidationReportV1 {
        let (protocol, inputs) = software_fixture_authority();
        evaluate_mhi_validation(&protocol, &inputs).expect("fixture report")
    }

    fn publish_bundle(
        output: &Path,
        report: &crate::results::MhiValidationReportV1,
        protocol_id: &str,
        overwrite: bool,
    ) -> Result<(), MhiValidationError> {
        let (protocol, inputs) = software_fixture_authority();
        assert_eq!(protocol.protocol_id, protocol_id);
        let authorization = authorize_publication(report, &protocol, &inputs)?;
        publish_authorized_bundle(output, &authorization, overwrite)
    }

    fn published_bundle_for_readdir_test(label: &str) -> (PathBuf, PathBuf) {
        let parent = temporary_parent(label);
        let output = parent.join("bundle");
        let report = software_fixture_report();
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("valid managed bundle");
        (parent, output)
    }

    fn readdir_call_count(directory: &fs::File, display_path: &Path) -> usize {
        set_publication_fault(Some(PublicationFault::ReadDirAt(usize::MAX)));
        read_child_names(directory, display_path).expect("normal directory enumeration");
        let calls = publication_read_dir_calls();
        set_publication_fault(None);
        calls
    }

    fn assert_readdir_io(error: MhiValidationError, expected_path: &Path) {
        match error {
            MhiValidationError::Io { path, source } => {
                assert_eq!(path.as_path(), expected_path);
                assert_eq!(source.raw_os_error(), Some(TEST_READDIR_ERRNO));
            }
            other => panic!("expected typed readdir I/O error, got {other:?}"),
        }
    }

    fn assert_managed_generation(path: &Path) {
        assert!(path.is_dir(), "managed generation remains a directory");
        assert!(
            path.join(REPORT_FILE).is_file(),
            "managed report remains present"
        );
        assert!(
            path.join(MANIFEST_FILE).is_file(),
            "managed manifest remains present"
        );
        assert!(
            path.join(SUMMARY_FILE).is_file(),
            "managed summary remains present"
        );
        assert!(
            path.join("tables").is_dir(),
            "managed tables remain present"
        );
    }

    fn assert_precommit_cleanup(parent: &Path, output: &Path) {
        assert!(
            !output.exists(),
            "pre-commit failure must not publish output"
        );
        assert!(
            !parent.join(".bundle.phase-e-stage").exists(),
            "pre-commit stage must be cleaned"
        );
        assert!(
            !parent.join(".bundle.phase-e-backup").exists(),
            "pre-commit backup must remain absent"
        );
    }

    fn ensure_persistent_lock(parent: &Path) {
        let authority = open_parent_authority(parent).expect("test parent authority");
        let lock = acquire_publication_lock(&authority, "bundle").expect("persistent test lock");
        drop(lock);
    }

    fn manifest(path: &Path) -> Value {
        serde_json::from_slice(
            &fs::read(path.join(MANIFEST_FILE)).expect("publication manifest bytes"),
        )
        .expect("publication manifest JSON")
    }

    fn assert_manifest_contract(path: &Path, mode: &str) {
        let manifest = manifest(path);
        assert_eq!(manifest["publication_mode"], mode);
        let records = manifest["generated_files"]
            .as_array()
            .expect("manifest generated files");
        assert_eq!(records.len(), 8);
        assert!(records.iter().all(|record| {
            record["relative_path"] != MANIFEST_FILE
                && record["relative_path"].is_string()
                && record["sha256"].as_str().is_some()
        }));
    }

    fn assert_bundle_matches_golden(path: &Path) {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase_e");
        let expected =
            fs::read_to_string(fixture_root.join("expected/golden_bundle_file_sha256s.txt"))
                .expect("golden digest list");
        let rows = expected
            .lines()
            .map(|line| {
                let mut columns = line.split('\t');
                (
                    columns.next().expect("golden path"),
                    columns.next().expect("golden length"),
                    columns.next().expect("golden hash"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 9);
        for (relative, length, digest) in rows {
            let actual = fs::read(path.join(relative)).expect("published golden file");
            assert_eq!(actual.len().to_string(), length, "{relative}");
            assert_eq!(
                format!("{:x}", Sha256::digest(&actual)),
                digest,
                "{relative}"
            );
            assert_eq!(
                actual,
                fs::read(fixture_root.join("expected/golden_bundle").join(relative))
                    .expect("golden bytes"),
                "{relative}"
            );
        }
    }

    fn error_targets_output(path: &Path, output: &Path) -> bool {
        path == output
    }

    fn publication_race_case(hook: PublicationTestHook) -> (PathBuf, PathBuf, MhiValidationError) {
        let parent = temporary_parent("generation_race");
        let output = parent.join("bundle");
        let report = software_fixture_report();
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("initial managed generation");
        set_publication_test_hook(Some(hook));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect_err("injected publication race");
        set_publication_test_hook(None);
        (parent, output, error)
    }

    #[test]
    fn phase_e_publication_is_locked_no_clobber_crash_durable_and_residue_exact() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let parent = temporary_parent("state_machine");
        let stage = parent.join(".bundle.phase-e-stage");
        let output = parent.join("bundle");
        fs::create_dir(&stage).expect("stage");
        fs::write(stage.join("generation.txt"), b"new").expect("stage bytes");
        fs::create_dir(&output).expect("existing output");
        fs::write(output.join("generation.txt"), b"old").expect("output bytes");

        let authority = open_parent_authority(&parent).expect("test parent authority");
        let lock = acquire_publication_lock(&authority, "bundle").expect("persistent lock");
        let lock_path = parent.join(".bundle.phase-e-publish.lock");
        assert!(lock_path.is_file());
        drop(lock);
        assert!(
            lock_path.is_file(),
            "lock identity must persist after release"
        );

        assert!(matches!(
            atomic_noreplace(&authority.descriptor, ".bundle.phase-e-stage", "bundle", &output),
            Err(MhiValidationError::PublicationConcurrentDestinationCreated { output: path })
                if path == output
        ));
        assert_eq!(
            fs::read(stage.join("generation.txt")).expect("stage remains"),
            b"new"
        );
        assert_eq!(
            fs::read(output.join("generation.txt")).expect("old output remains"),
            b"old"
        );

        atomic_exchange(
            &authority.descriptor,
            ".bundle.phase-e-stage",
            "bundle",
            &output,
        )
        .expect("atomic exchange");
        assert_eq!(
            fs::read(output.join("generation.txt")).expect("new output"),
            b"new"
        );
        assert_eq!(
            fs::read(stage.join("generation.txt")).expect("old stage"),
            b"old"
        );
        fs::remove_dir_all(parent).expect("cleanup");

        let report = software_fixture_report();
        for residue in [".bundle.phase-e-stage", ".bundle.phase-e-backup"] {
            let parent = temporary_parent("residue");
            let output = parent.join("bundle");
            fs::create_dir(parent.join(residue)).expect("pre-existing residue");
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("residue blocks publication");
            assert!(matches!(
                error,
                MhiValidationError::PublicationRecoveryResidue { output: ref path, .. }
                    if error_targets_output(path, &output)
            ));
            assert!(parent.join(residue).is_dir(), "residue remains intact");
            assert!(!output.exists(), "uncommitted output remains absent");
            fs::remove_dir_all(parent).expect("residue cleanup");
        }

        #[cfg(unix)]
        for link in [".bundle.phase-e-stage", ".bundle.phase-e-backup", "bundle"] {
            use std::os::unix::fs::symlink;

            let parent = temporary_parent("symlink");
            let output = parent.join("bundle");
            let link_path = parent.join(link);
            symlink("foreign-target", &link_path).expect("malicious symlink");
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("symlink blocks publication");
            if link == "bundle" {
                assert!(matches!(error, MhiValidationError::UnsafePath(_)));
            } else {
                assert!(matches!(
                    error,
                    MhiValidationError::PublicationRecoveryResidue { output: ref path, .. }
                        if error_targets_output(path, &output)
                ));
            }
            assert!(
                fs::symlink_metadata(&link_path)
                    .expect("symlink remains")
                    .file_type()
                    .is_symlink(),
                "publication never follows or removes a symlink"
            );
            fs::remove_dir_all(parent).expect("symlink cleanup");
        }

        let parent = temporary_parent("unmanaged");
        let output = parent.join("bundle");
        fs::create_dir(&output).expect("unmanaged output");
        fs::write(output.join("sentinel.txt"), b"do not clobber").expect("sentinel");
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect_err("unmanaged output is not replaceable");
        assert!(matches!(error, MhiValidationError::OutputNotManaged(_)));
        assert_eq!(
            fs::read(output.join("sentinel.txt")).expect("unmanaged sentinel"),
            b"do not clobber"
        );
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        fs::remove_dir_all(parent).expect("unmanaged cleanup");

        let (parent, output, error) =
            publication_race_case(PublicationTestHook::ReplaceOutputBeforePrecheck);
        assert!(
            matches!(
                error,
                MhiValidationError::PublicationConcurrentManagedOutputChanged { output: ref path, .. }
                    if error_targets_output(path, &output)
            ),
            "unexpected precheck error: {error:?}"
        );
        assert_eq!(
            fs::read(output.join("sentinel.txt")).expect("foreign output preserved"),
            b"foreign competitor"
        );
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        assert_managed_generation(&parent.join(".bundle.phase-e-foreign-competitor"));
        fs::remove_dir_all(parent).expect("precheck cleanup");

        let (parent, output, error) =
            publication_race_case(PublicationTestHook::ReplaceOutputBeforeExchange);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedForeignSwapDetected { output: ref path, .. }
                if error_targets_output(path, &output)
        ));
        assert_managed_generation(&output);
        assert_eq!(
            fs::read(parent.join(".bundle.phase-e-stage/sentinel.txt"))
                .expect("foreign generation retained at stage"),
            b"foreign competitor"
        );
        assert_managed_generation(&parent.join(".bundle.phase-e-foreign-competitor"));
        fs::remove_dir_all(parent).expect("pre-exchange cleanup");

        let (parent, output, error) =
            publication_race_case(PublicationTestHook::MutateOutputBeforeExchange);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedForeignSwapDetected { output: ref path, .. }
                if error_targets_output(path, &output)
        ));
        assert_managed_generation(&output);
        assert_eq!(
            fs::read(parent.join(".bundle.phase-e-stage").join(REPORT_FILE))
                .expect("mutated old generation retained"),
            b"mutated generation"
        );
        fs::remove_dir_all(parent).expect("same-inode old cleanup");

        let (parent, output, error) =
            publication_race_case(PublicationTestHook::ReplaceVisibleOutputAfterExchange);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedVisibleOutputChanged { output: ref path, .. }
                if error_targets_output(path, &output)
        ));
        assert_eq!(
            fs::read(output.join("sentinel.txt")).expect("visible competitor preserved"),
            b"foreign competitor"
        );
        assert_managed_generation(&parent.join(".bundle.phase-e-stage"));
        assert_managed_generation(&parent.join(".bundle.phase-e-foreign-competitor"));
        fs::remove_dir_all(parent).expect("post-exchange replacement cleanup");

        let (parent, output, error) =
            publication_race_case(PublicationTestHook::MutateVisibleOutputAfterExchange);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedVisibleOutputChanged { output: ref path, .. }
                if error_targets_output(path, &output)
        ));
        assert_eq!(
            fs::read(output.join(REPORT_FILE)).expect("mutated visible generation"),
            b"mutated generation"
        );
        assert_managed_generation(&parent.join(".bundle.phase-e-stage"));
        fs::remove_dir_all(parent).expect("post-exchange mutation cleanup");

        let (parent, output, error) =
            publication_race_case(PublicationTestHook::PrecreateBackupBeforeCleanup);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedCleanupFailed { output: ref path, .. }
                if error_targets_output(path, &output)
        ));
        assert_managed_generation(&output);
        assert_managed_generation(&parent.join(".bundle.phase-e-stage"));
        assert!(parent.join(".bundle.phase-e-backup").is_dir());
        fs::remove_dir_all(parent).expect("cleanup residue cleanup");
    }

    #[test]
    fn phase_e_publication_requires_real_replay_authorization() {
        let report = software_fixture_report();
        let (protocol, inputs) = software_fixture_authority();
        let mut forged = report.clone();
        forged.release_claims[0].statement.push_str(" forged");
        forged
            .validate_structure()
            .expect("claim mutation remains structurally valid");

        let parent = temporary_parent("replay_invalid");
        let output = parent.join("bundle");
        let error = authorize_publication(&forged, &protocol, &inputs)
            .expect_err("structural validity cannot authorize publication");
        assert!(
            matches!(error, MhiValidationError::Dataset(ref message) if message.contains("replay"))
        );
        assert!(!output.exists());
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        assert!(!parent.join(".bundle.phase-e-backup").exists());
        fs::remove_dir_all(parent).expect("replay-invalid cleanup");
    }

    #[test]
    fn phase_e_physical_looking_report_cannot_authorize_publication() {
        let report = software_fixture_report();
        let (protocol, inputs) = software_fixture_authority();
        let mut forged = report.clone();
        forged.release_claims[0].outcome =
            crate::validation_config::ReleaseClaimOutcomeV1::PhysicallyValidated;
        forged
            .validate_structure()
            .expect("physical-looking mutation remains structurally valid");

        let parent = temporary_parent("physical-looking-forgery");
        let output = parent.join("bundle");
        let error = authorize_publication(&forged, &protocol, &inputs)
            .expect_err("physical-looking report lacks production authority");
        assert!(matches!(error, MhiValidationError::Dataset(_)));
        assert!(!output.exists());
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        assert!(!parent.join(".bundle.phase-e-backup").exists());
        fs::remove_dir_all(parent).expect("physical-looking forgery cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn phase_e_parent_symlink_is_rejected_without_touching_target() {
        use std::os::unix::fs::symlink;

        let report = software_fixture_report();
        let real = temporary_parent("real_directory");
        let holder = temporary_parent("symlink_parent_holder");
        let symlink_parent = holder.join("symlink_parent");
        symlink(&real, &symlink_parent).expect("parent symlink");
        let output = symlink_parent.join("output");
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect_err("symlinked output parent is not publication authority");
        assert!(matches!(error, MhiValidationError::UnsafePath(path) if path == symlink_parent));
        assert!(!real.join("output").exists());
        assert!(!real.join(".output.phase-e-stage").exists());
        assert!(!real.join(".output.phase-e-backup").exists());
        fs::remove_dir_all(holder).expect("symlink holder cleanup");
        fs::remove_dir_all(real).expect("real target cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn phase_e_parent_namespace_replacement_uses_pinned_descriptor() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let report = software_fixture_report();
        let parent = temporary_parent("parent_namespace");
        let output = parent.join("bundle");
        set_publication_test_hook(Some(PublicationTestHook::ReplacePinnedParentPath));
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("publication remains inside the pinned parent");
        set_publication_test_hook(None);

        let foreign = parent.with_file_name(format!(
            ".{}.phase-e-foreign-parent",
            parent.file_name().expect("parent name").to_string_lossy()
        ));
        assert!(!parent.join("bundle").exists());
        assert_managed_generation(&foreign.join("bundle"));
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        assert!(!parent.join(".bundle.phase-e-backup").exists());
        fs::remove_dir_all(parent).expect("replacement parent cleanup");
        fs::remove_dir_all(foreign).expect("foreign parent cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn phase_e_held_generation_rejects_managed_file_symlink_substitution() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let report = software_fixture_report();
        let parent = temporary_parent("managed_file_symlink");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        set_publication_test_hook(Some(PublicationTestHook::ReplaceManagedFileWithSymlink));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect_err("descriptor-relative verification rejects a managed-file symlink");
        set_publication_test_hook(None);
        assert!(matches!(error, MhiValidationError::UnsafePath(_)));
        assert!(!output.exists());
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        assert_eq!(
            fs::read(parent.join(".phase-e-foreign-managed-report"))
                .expect("foreign managed file remains untouched"),
            fs::read(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/phase_e/expected/golden_bundle")
                    .join(REPORT_FILE),
            )
            .expect("golden report"),
        );
        fs::remove_dir_all(parent).expect("managed file symlink cleanup");
    }

    #[test]
    fn phase_e_readdir_root_error_is_not_eof() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let (parent, output) = published_bundle_for_readdir_test("readdir_root_error");
        let descriptor = open_directory_at_fd(
            AT_FDCWD,
            output.to_str().expect("managed path UTF-8"),
            &output,
        )
        .expect("managed root descriptor");
        let root_calls = readdir_call_count(&descriptor, &output);

        set_publication_fault(Some(PublicationFault::ReadDirAt(root_calls)));
        let error = verify_bundle_with_mode(&descriptor, &output, None)
            .expect_err("root readdir failure must reject exact verification");
        set_publication_fault(None);
        assert_readdir_io(error, &output);
        fs::remove_dir_all(parent).expect("root readdir error cleanup");
    }

    #[test]
    fn phase_e_readdir_tables_error_is_not_eof() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let (parent, output) = published_bundle_for_readdir_test("readdir_tables_error");
        let descriptor = open_directory_at_fd(
            AT_FDCWD,
            output.to_str().expect("managed path UTF-8"),
            &output,
        )
        .expect("managed root descriptor");
        let root_calls = readdir_call_count(&descriptor, &output);
        let tables_path = output.join("tables");
        let tables = open_directory_at_fd(descriptor.as_raw_fd(), "tables", &tables_path)
            .expect("tables descriptor");
        let tables_calls = readdir_call_count(&tables, &tables_path);

        set_publication_fault(Some(PublicationFault::ReadDirAt(root_calls + tables_calls)));
        let error = verify_bundle_with_mode(&descriptor, &output, None)
            .expect_err("tables readdir failure must reject exact verification");
        set_publication_fault(None);
        assert_readdir_io(error, &tables_path);
        fs::remove_dir_all(parent).expect("tables readdir error cleanup");
    }

    #[test]
    fn phase_e_readdir_normal_eof_accepts_exact_bundle_and_ignores_stale_errno() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let (parent, output) = published_bundle_for_readdir_test("readdir_normal_eof");
        let descriptor = open_directory_at_fd(
            AT_FDCWD,
            output.to_str().expect("managed path UTF-8"),
            &output,
        )
        .expect("managed root descriptor");

        set_errno_for_test(TEST_READDIR_ERRNO);
        verify_bundle_with_mode(&descriptor, &output, None)
            .expect("normal EOF with stale errno must accept exact bundle");
        fs::remove_dir_all(parent).expect("normal EOF cleanup");
    }

    #[test]
    fn phase_e_e_t22_complete_staging_and_byte_validation_matrix() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let report = software_fixture_report();

        let parent = temporary_parent("t22_success");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("create-new publication");
        assert_managed_generation(&output);
        assert_manifest_contract(&output, "create_new");
        assert_bundle_matches_golden(&output);
        let managed_descriptor = open_directory_at_fd(
            AT_FDCWD,
            output.to_str().expect("managed path UTF-8"),
            &output,
        )
        .expect("managed root descriptor");
        let root_entries =
            read_child_names(&managed_descriptor, &output).expect("managed root entries");
        assert_eq!(root_entries.len(), 4);
        assert_eq!(
            read_child_names(
                &open_directory_at_fd(
                    managed_descriptor.as_raw_fd(),
                    "tables",
                    &output.join("tables"),
                )
                .expect("tables descriptor"),
                &output.join("tables"),
            )
            .expect("table entries")
            .len(),
            6
        );
        fs::remove_dir_all(parent).expect("success cleanup");

        let parent = temporary_parent("t22_replace");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("initial replacement generation");
        publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect("managed replacement");
        assert_manifest_contract(&output, "replace_managed_bundle");
        fs::remove_dir_all(parent).expect("replacement cleanup");

        let parent = temporary_parent("t22_wrong_replace_mode");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("initial generation for mode mutation");
        set_publication_test_hook(Some(PublicationTestHook::WrongReplaceManifestMode));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect_err("wrong replacement manifest mode");
        set_publication_test_hook(None);
        assert!(matches!(error, MhiValidationError::Dataset(_)));
        assert_managed_generation(&output);
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        fs::remove_dir_all(parent).expect("wrong replacement mode cleanup");

        for ordinal in 1..=10 {
            let parent = temporary_parent("t22_write_failure");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            set_publication_fault(Some(PublicationFault::WriteAt(ordinal)));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("each generated write failure is rejected");
            set_publication_fault(None);
            assert!(matches!(error, MhiValidationError::Io { .. }));
            assert_precommit_cleanup(&parent, &output);
            fs::remove_dir_all(parent).expect("write failure cleanup");
        }

        for ordinal in 1..=9 {
            let parent = temporary_parent("t22_file_fsync_failure");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            set_publication_fault(Some(PublicationFault::SyncFileAt(ordinal)));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("each generated file fsync failure is rejected");
            set_publication_fault(None);
            assert!(matches!(error, MhiValidationError::Io { .. }));
            assert_precommit_cleanup(&parent, &output);
            fs::remove_dir_all(parent).expect("file fsync failure cleanup");
        }

        let parent = temporary_parent("t22_lock_fsync_failure");
        let output = parent.join("bundle");
        set_publication_fault(Some(PublicationFault::SyncFileAt(1)));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect_err("lock-file fsync failure");
        set_publication_fault(None);
        assert!(matches!(error, MhiValidationError::Io { .. }));
        assert!(!output.exists());
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        assert!(parent.join(".bundle.phase-e-publish.lock").is_file());
        fs::remove_dir_all(parent).expect("lock fsync failure cleanup");

        for ordinal in 1..=4 {
            let parent = temporary_parent("t22_directory_fsync_failure");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            set_publication_fault(Some(PublicationFault::SyncDirectoryAt(ordinal)));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("each staging directory fsync failure is rejected");
            set_publication_fault(None);
            assert!(matches!(error, MhiValidationError::Io { .. }));
            assert_precommit_cleanup(&parent, &output);
            fs::remove_dir_all(parent).expect("directory fsync failure cleanup");
        }

        let staged_mutations = [
            PublicationTestHook::MutateStagedChecksum,
            PublicationTestHook::MutateStagedReportBeforeReread,
            PublicationTestHook::AddManifestSelfRecord,
            PublicationTestHook::WrongCreateManifestMode,
            PublicationTestHook::AddManifestTimestamp,
            PublicationTestHook::AddManifestUnknownField,
            PublicationTestHook::AddExtraGeneratedFile,
            PublicationTestHook::RemoveManagedFile,
        ];
        for hook in staged_mutations {
            let parent = temporary_parent("t22_staged_mutation");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            set_publication_test_hook(Some(hook));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("invalid staged bundle is rejected");
            set_publication_test_hook(None);
            assert!(!matches!(
                error,
                MhiValidationError::PublicationDurabilityUnconfirmed { .. }
            ));
            assert_precommit_cleanup(&parent, &output);
            fs::remove_dir_all(parent).expect("staged mutation cleanup");
        }

        for fault in [
            PublicationFault::NoReplaceUnsupported,
            PublicationFault::NoReplaceFailure,
        ] {
            let parent = temporary_parent("t22_noreplace_failure");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            set_publication_fault(Some(fault));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("no-replace failure");
            set_publication_fault(None);
            assert!(matches!(
                error,
                MhiValidationError::UnsupportedAtomicPublicationFilesystem(_)
                    | MhiValidationError::Io { .. }
            ));
            assert_precommit_cleanup(&parent, &output);
            fs::remove_dir_all(parent).expect("no-replace failure cleanup");
        }

        for fault in [
            PublicationFault::ExchangeUnsupported,
            PublicationFault::ExchangeFailure,
        ] {
            let parent = temporary_parent("t22_exchange_failure");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect("old managed generation");
            let old_report = fs::read(output.join(REPORT_FILE)).expect("old report");
            set_publication_fault(Some(fault));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
                .expect_err("exchange failure");
            set_publication_fault(None);
            assert!(matches!(
                error,
                MhiValidationError::UnsupportedAtomicPublicationFilesystem(_)
                    | MhiValidationError::Io { .. }
            ));
            assert_eq!(
                fs::read(output.join(REPORT_FILE)).expect("old report remains"),
                old_report
            );
            assert!(
                !parent.join(".bundle.phase-e-stage").exists(),
                "failed exchange cleans new stage"
            );
            fs::remove_dir_all(parent).expect("exchange failure cleanup");
        }

        let parent = temporary_parent("t22_create_fsync_after_commit");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        set_publication_fault(Some(PublicationFault::SyncDirectoryAt(5)));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect_err("create durability is unconfirmed");
        set_publication_fault(None);
        assert!(matches!(
            error,
            MhiValidationError::PublicationDurabilityUnconfirmed {
                operation: "create_new",
                ..
            }
        ));
        assert_managed_generation(&output);
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        fs::remove_dir_all(parent).expect("create durability cleanup");

        let parent = temporary_parent("t22_replace_fsync_after_exchange");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("old generation");
        set_publication_fault(Some(PublicationFault::SyncDirectoryAt(5)));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect_err("replacement durability is unconfirmed");
        set_publication_fault(None);
        assert!(matches!(
            error,
            MhiValidationError::PublicationDurabilityUnconfirmed {
                operation: "replace_managed_bundle",
                ..
            }
        ));
        assert_managed_generation(&output);
        assert_managed_generation(&parent.join(".bundle.phase-e-stage"));
        fs::remove_dir_all(parent).expect("replacement durability cleanup");
    }

    #[test]
    fn phase_e_e_t23_lock_holder_process() {
        let Some(parent) = std::env::var_os("PHASE_E_LOCK_HOLDER_PARENT") else {
            return;
        };
        let parent = PathBuf::from(parent);
        let ready = PathBuf::from(
            std::env::var_os("PHASE_E_LOCK_HOLDER_READY").expect("lock holder ready path"),
        );
        let release = PathBuf::from(
            std::env::var_os("PHASE_E_LOCK_HOLDER_RELEASE").expect("lock holder release path"),
        );
        let authority = open_parent_authority(&parent).expect("child parent authority");
        let lock = acquire_publication_lock(&authority, "bundle").expect("child holds lock");
        fs::write(&ready, b"ready").expect("lock holder readiness");
        while !release.exists() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        drop(lock);
    }

    #[test]
    fn phase_e_e_t23_true_lock_contention_concurrent_create_and_recovery_matrix() {
        let _serial = PUBLICATION_TEST_SERIAL
            .lock()
            .expect("publication test serialization");
        let report = software_fixture_report();

        let parent = temporary_parent("t23_lock_contention");
        let output = parent.join("bundle");
        let ready = parent.join("ready");
        let release = parent.join("release");
        let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "mhi_validation::output::tests::phase_e_e_t23_lock_holder_process",
                "--nocapture",
            ])
            .env("PHASE_E_LOCK_HOLDER_PARENT", &parent)
            .env("PHASE_E_LOCK_HOLDER_READY", &ready)
            .env("PHASE_E_LOCK_HOLDER_RELEASE", &release)
            .spawn()
            .expect("spawn independent lock holder");
        for _ in 0..200 {
            if ready.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(
            ready.exists(),
            "first publisher must hold the persistent lock"
        );
        let contention = publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect_err("second publisher must observe lock contention");
        assert!(matches!(
            contention,
            MhiValidationError::PublicationLocked(_)
        ));
        assert!(!output.exists());
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        assert!(!parent.join(".bundle.phase-e-backup").exists());
        fs::write(&release, b"release").expect("release child lock");
        assert!(child.wait().expect("wait for lock holder").success());
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("publisher completes after lock release");
        assert_managed_generation(&output);
        fs::remove_dir_all(parent).expect("lock contention cleanup");

        let parent = temporary_parent("t23_concurrent_create");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        set_publication_test_hook(Some(PublicationTestHook::CreateOutputBeforeCommit));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect_err("concurrent create after preflight");
        set_publication_test_hook(None);
        assert!(matches!(
            error,
            MhiValidationError::PublicationConcurrentDestinationCreated { output: ref path }
                if error_targets_output(path, &output)
        ));
        assert_eq!(
            fs::read(output.join("sentinel.txt")).expect("competitor output"),
            b"concurrent competitor"
        );
        assert!(!parent.join(".bundle.phase-e-stage").exists());
        fs::remove_dir_all(parent).expect("concurrent create cleanup");

        for residue in [".bundle.phase-e-stage", ".bundle.phase-e-backup"] {
            let parent = temporary_parent("t23_residue");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            fs::create_dir(parent.join(residue)).expect("private residue");
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect_err("residue blocks new publication");
            assert!(matches!(
                error,
                MhiValidationError::PublicationRecoveryResidue {
                    stage_state: _,
                    backup_state: _,
                    remaining_paths: ref paths,
                    ..
                } if paths.iter().any(|path| path.ends_with(residue))
            ));
            assert!(parent.join(residue).is_dir());
            assert!(!output.exists());
            fs::remove_dir_all(parent).expect("residue cleanup");
        }

        set_publication_test_hook(None);
        for hook in [
            PublicationTestHook::MutateOldStageBeforeProof,
            PublicationTestHook::PrecreateBackupBeforeCleanup,
        ] {
            let parent = temporary_parent("t23_generation_proof");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect("old managed generation");
            set_publication_test_hook(Some(hook));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
                .expect_err("generation proof or cleanup failure");
            set_publication_test_hook(None);
            assert!(matches!(
                error,
                MhiValidationError::PublicationCommittedForeignSwapDetected { .. }
                    | MhiValidationError::PublicationCommittedCleanupFailed { .. }
            ));
            assert_managed_generation(&output);
            fs::remove_dir_all(parent).expect("generation proof cleanup");
        }

        let parent = temporary_parent("t23_stage_to_backup");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("old managed generation");
        set_publication_fault(Some(PublicationFault::NoReplaceFailure));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect_err("stage-to-backup failure");
        set_publication_fault(None);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedCleanupFailed { .. }
        ));
        assert_managed_generation(&output);
        assert_managed_generation(&parent.join(".bundle.phase-e-stage"));
        fs::remove_dir_all(parent).expect("stage-to-backup cleanup");

        for ordinal in 1..=11 {
            let parent = temporary_parent("t23_reverse_delete");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect("old managed generation");
            set_publication_fault(Some(PublicationFault::DeleteAt(ordinal)));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
                .expect_err("reverse deletion failure");
            set_publication_fault(None);
            assert!(matches!(
                error,
                MhiValidationError::PublicationCommittedCleanupFailed { .. }
            ));
            assert_managed_generation(&output);
            fs::remove_dir_all(parent).expect("reverse deletion cleanup");
        }

        for ordinal in 1..=18 {
            let parent = temporary_parent("t23_every_directory_fsync");
            let output = parent.join("bundle");
            ensure_persistent_lock(&parent);
            publish_bundle(&output, &report, "phase_e_software_protocol", false)
                .expect("old managed generation");
            set_publication_fault(Some(PublicationFault::SyncDirectoryAt(ordinal)));
            let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
                .expect_err("directory fsync failure at every state-machine point");
            set_publication_fault(None);
            if ordinal <= 4 {
                assert!(matches!(error, MhiValidationError::Io { .. }));
            } else if ordinal == 5 {
                assert!(matches!(
                    error,
                    MhiValidationError::PublicationDurabilityUnconfirmed {
                        operation: "replace_managed_bundle",
                        ..
                    }
                ));
            } else {
                assert!(matches!(
                    error,
                    MhiValidationError::PublicationCommittedCleanupFailed { .. }
                ));
            }
            assert_managed_generation(&output);
            fs::remove_dir_all(parent).expect("directory fsync matrix cleanup");
        }

        let parent = temporary_parent("t23_cleanup_dir_fsync");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("old managed generation");
        set_publication_fault(Some(PublicationFault::SyncDirectoryAt(7)));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect_err("directory fsync after backup deletion");
        set_publication_fault(None);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedCleanupFailed { .. }
        ));
        assert_managed_generation(&output);
        fs::remove_dir_all(parent).expect("directory fsync cleanup");

        let parent = temporary_parent("t23_old_replacement");
        let output = parent.join("bundle");
        ensure_persistent_lock(&parent);
        publish_bundle(&output, &report, "phase_e_software_protocol", false)
            .expect("old managed generation");
        set_publication_test_hook(Some(PublicationTestHook::ReplaceOutputBeforeExchange));
        let error = publish_bundle(&output, &report, "phase_e_software_protocol", true)
            .expect_err("old generation replaced before exchange");
        set_publication_test_hook(None);
        assert!(matches!(
            error,
            MhiValidationError::PublicationCommittedForeignSwapDetected { .. }
        ));
        assert_managed_generation(&output);
        assert_managed_generation(&parent.join(".bundle.phase-e-foreign-competitor"));
        assert_eq!(
            fs::read(parent.join(".bundle.phase-e-stage/sentinel.txt"))
                .expect("foreign competitor retained at stage"),
            b"foreign competitor"
        );
        fs::remove_dir_all(parent).expect("old replacement cleanup");
    }
}
