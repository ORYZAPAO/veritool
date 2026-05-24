use crate::args::OutputFormat;

pub fn format_ports_csv(_module: &crate::design::Module) -> anyhow::Result<()> {
    Ok(())
}

pub fn format_signals_csv(_module: &crate::design::Module) -> anyhow::Result<()> {
    Ok(())
}

pub fn format_ff_csv(_module: &crate::design::Module) -> anyhow::Result<()> {
    Ok(())
}

pub fn format_hier_csv(_design: &crate::design::Design, _top_module: &str) -> anyhow::Result<()> {
    Ok(())
}

pub fn format_top_csv(_top_modules: &[String]) -> anyhow::Result<()> {
    Ok(())
}

pub fn format_report_csv(_design: &crate::design::Design, _top_module: &str) -> anyhow::Result<()> {
    Ok(())
}
