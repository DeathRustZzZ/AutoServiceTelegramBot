pub fn main_menu() -> &'static str {
    "Главное меню. Выберите раздел на нижней панели."
}

pub fn not_implemented(section: &str) -> String {
    format!("{section}\n\nЭтот раздел будет добавлен на следующих этапах.")
}
