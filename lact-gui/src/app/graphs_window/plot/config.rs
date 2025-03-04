use crate::app::graphs_window::stat::StatType;

#[derive(Default, Clone, Debug)]
pub struct PlotConfig {
    pub left_stats: Vec<StatType>,
    pub right_stats: Vec<StatType>,
}
