#[derive(Debug)]
pub struct MainShutdown {}

#[derive(Debug)]
pub enum MainRecv {
    SelectTab(String),
    ListTabs,
    SelectInteractive,
    CloseTab(String),
    AutocompleteTab(String),
}
