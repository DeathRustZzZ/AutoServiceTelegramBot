pub fn main_menu() -> &'static str {
    "Главное меню. Используйте нижнюю клавиатуру для выбора раздела."
}

pub fn not_implemented(section: &str) -> String {
    format!("{section}\n\nЭтот раздел будет добавлен на следующих этапах.")
}
