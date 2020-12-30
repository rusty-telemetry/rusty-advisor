use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;

use crate::errors::Result;
use crate::exporters::prometheus_exporter::metrics::prometheus_histogram::PrometheusHistogram;
use crate::metrics::metric::MetricDescription;

pub fn encode_histogram<W: Write>(histogram: &PrometheusHistogram, writer: &mut W) -> Result<()> {
    let metric_description = histogram.metric_description();
    let name = metric_description.name();
    let help = metric_description.description();

    if !help.is_empty() {
        writeln!(writer, "# HELP {} {}", name, escape_string(help, false))?;
    }
    writeln!(writer, "# TYPE {} histogram", name)?;

    for (i, bucket) in histogram.buckets().iter().enumerate() {
        let bucket_bound = bucket.0.to_string();
        let bucket_bound = if i == histogram.buckets().len() - 1 { "+Inf" } else { bucket_bound.borrow() };
        let bucket_value = bucket.1;

        write_sample(
            format!("{}_bucket", name).borrow(),
            metric_description,
            vec!(("le", bucket_bound)),
            bucket_value,
            Some(histogram.timestamp_ms()),
            writer,
        )?;
    }

    write_sample(
        &format!("{}_sum", name),
        metric_description,
        vec!(),
        histogram.sum(),
        Some(histogram.timestamp_ms()),
        writer,
    )?;

    write_sample(
        &format!("{}_count", name),
        metric_description,
        vec!(),
        histogram.count(),
        Some(histogram.timestamp_ms()),
        writer,
    )?;

    Ok(())
}

fn write_sample<V>(
    name: &str,
    metric_description: &MetricDescription,
    additional_labels: Vec<(&str, &str)>,
    value: V,
    timestamp: Option<u64>,
    writer: &mut dyn Write,
) -> Result<()>
    where V: Display
{
    writer.write_all(name.as_bytes())?;

    add_label_pairs(
        metric_description.tags(),
        &additional_labels,
        writer,
    )?;

    write!(writer, " {}", value)?;

    if let Some(ts) = timestamp {
        write!(writer, " {}", ts)?;
    }
    // let timestamp = timestamp_ms;
    // if timestamp != 0 {
    // }

    writer.write_all(b"\n")?;

    Ok(())
}

fn add_label_pairs(
    tags: &HashMap<String, String>,
    additional_labels: &Vec<(&str, &str)>,
    writer: &mut dyn Write,
) -> Result<()> {
    if tags.is_empty() && additional_labels.is_empty() {
        return Ok(());
    }

    let mut separator = "{";
    for label in tags {
        let label_name = label.0;
        let label_value = label.1;
        write!(
            writer,
            "{}{}=\"{}\"",
            separator,
            label_name,
            escape_string(label_value, true)
        )?;

        separator = ",";
    }

    if !additional_labels.is_empty() {
        for extra_label in additional_labels {
            let label_name = extra_label.0;
            let label_value = extra_label.1;
            write!(
                writer,
                "{}{}=\"{}\"",
                separator,
                label_name,
                escape_string(label_value, true)
            )?;
        }
    }

    writer.write_all(b"}")?;

    Ok(())
}

/// Replaces `\` by `\\`, new line character by `\n`, and `"` by `\"` if
/// `include_double_quote` is true.
fn escape_string(v: &str, include_double_quote: bool) -> String {
    let mut escaped = String::with_capacity(v.len() * 2);

    for c in v.chars() {
        match c {
            '\\' | '\n' => {
                escaped.extend(c.escape_default());
            }
            '"' if include_double_quote => {
                escaped.extend(c.escape_default());
            }
            _ => {
                escaped.push(c);
            }
        }
    }

    escaped.shrink_to_fit();

    escaped
}
