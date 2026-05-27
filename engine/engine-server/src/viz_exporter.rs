use engine_core::{D3Link, D3Node, VisualizationManifest};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EmbeddedVisualizationExporter {
    output_root: PathBuf,
}

impl EmbeddedVisualizationExporter {
    pub fn new(output_root: impl Into<PathBuf>) -> Self {
        Self {
            output_root: output_root.into(),
        }
    }

    pub fn export_network_graph(&self, graph_id: &str) -> io::Result<VisualizationManifest> {
        fs::create_dir_all(&self.output_root)?;

        let manifest = default_network_graph_manifest(graph_id);
        let filename = format!("{}.json", sanitize_file_stem(graph_id));
        let output_path = self.output_root.join(filename);
        fs::write(output_path, serde_json::to_vec_pretty(&manifest)?)?;

        Ok(manifest)
    }

    pub fn output_root(&self) -> &Path {
        &self.output_root
    }
}

impl Default for EmbeddedVisualizationExporter {
    fn default() -> Self {
        Self::new(PathBuf::from("kernel_store").join("visualizations"))
    }
}

fn default_network_graph_manifest(graph_id: &str) -> VisualizationManifest {
    VisualizationManifest::new(
        graph_id.to_string(),
        chrono::Utc::now().timestamp_millis(),
        vec![
            D3Node::new(
                "engine-core".to_string(),
                "Core primitives".to_string(),
                "core".to_string(),
                1.0,
            ),
            D3Node::new(
                "engine-ml".to_string(),
                "Inference controllers".to_string(),
                "ml".to_string(),
                1.0,
            ),
            D3Node::new(
                "engine-server".to_string(),
                "Routing and export".to_string(),
                "server".to_string(),
                1.0,
            ),
        ],
        vec![
            D3Link::new(
                "engine-server".to_string(),
                "engine-core".to_string(),
                "reads manifest schema".to_string(),
                1.0,
            ),
            D3Link::new(
                "engine-server".to_string(),
                "engine-ml".to_string(),
                "routes inference graph".to_string(),
                1.0,
            ),
        ],
    )
}

fn sanitize_file_stem(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            '/' | '\\' | '?' | '&' | '=' | ':' | ';' | '#' | '%' => '_',
            ch if ch.is_whitespace() => '_',
            _ => '_',
        })
        .collect();

    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        "visualization".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{sanitize_file_stem, EmbeddedVisualizationExporter};
    use std::fs;

    #[test]
    fn sanitizer_replaces_toxic_path_and_query_characters() {
        assert_eq!(
            sanitize_file_stem("../network/graph?tenant=a&mode=embed"),
            ".._network_graph_tenant_a_mode_embed"
        );
    }

    #[test]
    fn exporter_writes_manifest_inside_output_root() {
        let root = std::env::temp_dir().join(format!("grove-viz-exporter-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let exporter = EmbeddedVisualizationExporter::new(&root);
        let manifest = exporter
            .export_network_graph("../network/graph?tenant=a")
            .expect("export should succeed");
        let output = root.join(".._network_graph_tenant_a.json");

        assert_eq!(manifest.graph_id, "../network/graph?tenant=a");
        assert!(output.exists());
        assert!(output.starts_with(exporter.output_root()));

        let _ = fs::remove_dir_all(&root);
    }
}
