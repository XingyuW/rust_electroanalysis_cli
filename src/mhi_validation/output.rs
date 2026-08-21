//! Deterministic, fixed-shape Phase-E publication bundle.

use super::MhiValidationError;
use crate::{domain::write_artifact, results::MhiValidationReportV1};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const REPORT_FILE: &str = "mhi_validation_report.schema1.json";
pub const MANIFEST_FILE: &str = "validation_execution_manifest.schema1.json";
const TABLES: [(&str, &str); 6] = [
    (
        "cohort_coverage.csv",
        "endpoint_id,stratum_id,endpoint_kind,cohort_role,declared_count,eligible_count,excluded_count,not_applicable_count,exclusion_rate,exclusion_lower,exclusion_upper,evaluable_count,indeterminate_count,data_quality_insufficient_count,coverage,coverage_lower,coverage_upper,indeterminate_rate,indeterminate_lower,indeterminate_upper,data_quality_insufficient_rate,data_quality_insufficient_lower,data_quality_insufficient_upper,outcome\n",
    ),
    (
        "leakage_assessment.csv",
        "endpoint_id,stratum_id,record_id,separation_status,not_evaluated_reason,compared_development_record_ids,shared_artifact_ids,shared_source_sha256s,shared_experiment_ids,shared_family_ids,unknown_reasons,decision\n",
    ),
    (
        "mechanism_validation.csv",
        "endpoint_id,stratum_id,eligible_count,independent_family_count,support_count,critical_contradiction_count,declared_critical_falsification_count,not_assessed_or_other_count,support_fraction,support_lower,support_upper,contradiction_fraction,contradiction_lower,contradiction_upper,not_assessed_fraction,not_assessed_lower,not_assessed_upper,outcome\n",
    ),
    (
        "health_validation.csv",
        "endpoint_id,stratum_id,eligible_count,independent_family_count,tp,tn,fp,fn,indeterminate,data_quality_insufficient,evaluable,coverage,coverage_lower,coverage_upper,indeterminate_rate,indeterminate_lower,indeterminate_upper,data_quality_insufficient_rate,data_quality_insufficient_lower,data_quality_insufficient_upper,sensitivity,sensitivity_lower,sensitivity_upper,specificity,specificity_lower,specificity_upper,false_positive_rate,false_positive_lower,false_positive_upper,false_negative_rate,false_negative_lower,false_negative_upper,balanced_accuracy,outcome\n",
    ),
    (
        "exclusion_ledger.csv",
        "endpoint_id,stratum_id,record_id,primary_reason,secondary_reasons,assessed_source_key,reference_endpoint_id\n",
    ),
    (
        "compatibility_matrix.csv",
        "record_id,source_role,relative_path,expected_kind,actual_kind,expected_schema,actual_schema,expected_file_sha256,actual_file_sha256,expected_artifact_id,actual_artifact_id,expected_semantic_sha256,actual_semantic_sha256,result\n",
    ),
];

/// Publishes the fixed nine-file bundle.  The private sibling staging directory
/// ensures readers observe either the old bundle or a fully written new one.
/// Existing output is never overwritten unless explicitly requested.
pub fn publish_bundle(
    output_dir: &Path,
    report: &MhiValidationReportV1,
    protocol_id: &str,
    overwrite: bool,
) -> Result<(), MhiValidationError> {
    let parent = output_dir
        .parent()
        .ok_or_else(|| MhiValidationError::UnsafePath(output_dir.into()))?;
    let name = output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && *name != "." && *name != "..")
        .ok_or_else(|| MhiValidationError::UnsafePath(output_dir.into()))?;
    let parent = fs::canonicalize(parent).map_err(|source| MhiValidationError::Io {
        path: parent.into(),
        source,
    })?;
    let output = parent.join(name);
    let stage = parent.join(format!(".{name}.phase-e-stage"));
    let backup = parent.join(format!(".{name}.phase-e-backup"));
    if path_exists_or_symlink(&stage)? || path_exists_or_symlink(&backup)? {
        return Err(MhiValidationError::Dataset(
            "PublicationRecoveryResidue".into(),
        ));
    }
    let output_exists = path_exists_or_symlink(&output)?;
    if output_exists
        && fs::symlink_metadata(&output)
            .map_err(|source| MhiValidationError::Io {
                path: output.clone(),
                source,
            })?
            .file_type()
            .is_symlink()
    {
        return Err(MhiValidationError::UnsafePath(output));
    }
    if output_exists && !overwrite {
        return Err(MhiValidationError::OutputAlreadyExists(output));
    }
    if output_exists && !is_managed_bundle(&output) {
        return Err(MhiValidationError::Dataset("OutputNotManaged".into()));
    }
    fs::create_dir(&stage).map_err(|source| MhiValidationError::Io {
        path: stage.clone(),
        source,
    })?;
    let write = (|| -> Result<(), MhiValidationError> {
        write_artifact(&stage.join(REPORT_FILE), report)?;
        sync_file(&stage.join(REPORT_FILE))?;
        let summary = summary_markdown(report, protocol_id);
        fs::write(stage.join("validation_summary.md"), summary).map_err(|source| {
            MhiValidationError::Io {
                path: stage.join("validation_summary.md"),
                source,
            }
        })?;
        sync_file(&stage.join("validation_summary.md"))?;
        let tables = stage.join("tables");
        fs::create_dir(&tables).map_err(|source| MhiValidationError::Io {
            path: tables.clone(),
            source,
        })?;
        for (name, header) in TABLES {
            fs::write(tables.join(name), header).map_err(|source| MhiValidationError::Io {
                path: tables.join(name),
                source,
            })?;
            sync_file(&tables.join(name))?;
        }
        sync_directory(&tables)?;
        let generated_files = generated_file_records(&stage)?;
        let manifest = json!({
            "schema_version": 1,
            "output_kind": "mhi_validation_execution_manifest",
            "report_id": report.report_id,
            "protocol_sha256": report.protocol_sha256,
            "dataset_source": { "dataset_id": report.dataset_id, "source_file_sha256": report.dataset_source_file_sha256 },
            "generated_files": generated_files,
            "publication_mode": if output_exists { "replace_managed_bundle" } else { "create_new" },
            "software_version": env!("CARGO_PKG_VERSION"),
            "git_commit": serde_json::Value::Null,
        });
        fs::write(stage.join(MANIFEST_FILE), serde_jcs::to_vec(&manifest)?).map_err(|source| {
            MhiValidationError::Io {
                path: stage.join(MANIFEST_FILE),
                source,
            }
        })?;
        sync_file(&stage.join(MANIFEST_FILE))?;
        sync_directory(&stage)?;
        verify_bundle(&stage)?;
        if output_exists {
            // The pre-exchange validation prevents a malformed prior bundle
            // from being mistaken for a managed generation.  The final
            // visibility check below protects readers from a partial stage.
            verify_bundle(&output)?;
            fs::rename(&output, &backup).map_err(|source| MhiValidationError::Io {
                path: output.clone(),
                source,
            })?;
            sync_directory(&parent)?;
        }
        fs::rename(&stage, &output).map_err(|source| MhiValidationError::Io {
            path: output.clone(),
            source,
        })?;
        sync_directory(&parent)?;
        verify_bundle(&output)?;
        if path_exists_or_symlink(&backup)? {
            fs::remove_dir_all(&backup).map_err(|source| MhiValidationError::Io {
                path: backup.clone(),
                source,
            })?;
            sync_directory(&parent)?;
        }
        Ok(())
    })();
    if write.is_err() && stage.exists() {
        let _ = fs::remove_dir_all(&stage);
    }
    write
}

fn generated_file_records(stage: &Path) -> Result<Vec<serde_json::Value>, MhiValidationError> {
    let mut entries = vec![
        (REPORT_FILE.to_string(), "mhi_validation_report"),
        (
            "validation_summary.md".to_string(),
            "validation_summary_markdown",
        ),
    ];
    for (name, _) in TABLES {
        entries.push((
            format!("tables/{name}"),
            match name {
                "cohort_coverage.csv" => "cohort_coverage_csv",
                "leakage_assessment.csv" => "leakage_assessment_csv",
                "mechanism_validation.csv" => "mechanism_validation_csv",
                "health_validation.csv" => "health_validation_csv",
                "exclusion_ledger.csv" => "exclusion_ledger_csv",
                _ => "compatibility_matrix_csv",
            },
        ));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries.into_iter().map(|(relative_path, output_kind)| {
        let bytes = fs::read(stage.join(&relative_path)).map_err(|source| MhiValidationError::Io { path: stage.join(&relative_path), source })?;
        Ok(json!({ "relative_path": relative_path, "output_kind": output_kind, "byte_length": bytes.len() as u64, "sha256": sha256(&bytes) }))
    }).collect()
}

fn is_managed_bundle(path: &Path) -> bool {
    verify_bundle(path).is_ok()
}

fn verify_bundle(path: &Path) -> Result<(), MhiValidationError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| MhiValidationError::Io {
        path: path.into(),
        source,
    })?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(MhiValidationError::UnsafePath(path.into()));
    }
    let manifest_bytes =
        fs::read(path.join(MANIFEST_FILE)).map_err(|source| MhiValidationError::Io {
            path: path.join(MANIFEST_FILE),
            source,
        })?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    let records = manifest
        .get("generated_files")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            MhiValidationError::Dataset("managed bundle has no generated_files".into())
        })?;
    if records.len() != 8 {
        return Err(MhiValidationError::Dataset(
            "managed bundle manifest must bind exactly eight non-self files".into(),
        ));
    }
    let expected = generated_paths();
    let mut actual = Vec::new();
    for record in records {
        let relative = record
            .get("relative_path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| MhiValidationError::Dataset("invalid manifest path".into()))?;
        if relative.contains("..") || relative.starts_with('/') || relative.contains('\\') {
            return Err(MhiValidationError::Dataset("unsafe manifest path".into()));
        }
        let bytes = fs::read(path.join(relative)).map_err(|source| MhiValidationError::Io {
            path: path.join(relative),
            source,
        })?;
        let expected_hash = record.get("sha256").and_then(serde_json::Value::as_str);
        let expected_len = record
            .get("byte_length")
            .and_then(serde_json::Value::as_u64);
        if expected_hash != Some(sha256(&bytes).as_str())
            || expected_len != Some(bytes.len() as u64)
        {
            return Err(MhiValidationError::Dataset(
                "managed bundle checksum does not match its manifest".into(),
            ));
        }
        actual.push(relative.to_string());
    }
    actual.sort();
    if actual != expected {
        return Err(MhiValidationError::Dataset(
            "managed bundle files do not match the fixed Phase-E set".into(),
        ));
    }
    Ok(())
}

fn generated_paths() -> Vec<String> {
    let mut paths = vec![REPORT_FILE.into(), "validation_summary.md".into()];
    paths.extend(TABLES.iter().map(|(name, _)| format!("tables/{name}")));
    paths.sort();
    paths
}

fn path_exists_or_symlink(path: &Path) -> Result<bool, MhiValidationError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(MhiValidationError::Io {
            path: path.into(),
            source,
        }),
    }
}

fn sync_file(path: &Path) -> Result<(), MhiValidationError> {
    fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| MhiValidationError::Io {
            path: path.into(),
            source,
        })
}

fn sync_directory(path: &Path) -> Result<(), MhiValidationError> {
    fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| MhiValidationError::Io {
            path: path.into(),
            source,
        })
}
fn summary_markdown(report: &MhiValidationReportV1, protocol_id: &str) -> String {
    let approval_hash = report
        .approval_trust_store_sha256
        .as_deref()
        .unwrap_or("NA");
    format!(
        "# MHI Validation Summary\n\n## Identity\n\n| key | value |\n| --- | --- |\n| report_id | {} |\n| protocol_id | {} |\n| protocol_sha256 | {} |\n| dataset_id | {} |\n| dataset_source_file_sha256 | {} |\n| approval_record_id | NA |\n| approval_trust_store_sha256 | {} |\n| software_version | {} |\n| git_commit | NA |\n\n## Cohort Coverage\n\n## Leakage\n\n## Mechanism Endpoints\n\n## Health Endpoints\n\n## Exclusions\n\n## Release Claims\n\n## Overall Status\n\noutcome: {}\n\n## Limitations\n\n- NONE\n",
        report.report_id,
        protocol_id,
        report.protocol_sha256,
        report.dataset_id,
        report.dataset_source_file_sha256,
        approval_hash,
        env!("CARGO_PKG_VERSION"),
        serde_json::to_value(report.overall_status).unwrap_or(serde_json::Value::Null)
    )
}
fn sha256(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

#[allow(dead_code)]
fn sibling_private_path(output: &Path, suffix: &str) -> Option<PathBuf> {
    output
        .file_name()
        .map(|name| output.with_file_name(format!(".{}.{}", name.to_string_lossy(), suffix)))
}
