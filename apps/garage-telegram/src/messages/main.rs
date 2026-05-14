pub fn welcome() -> &'static str {
    "Глобальная навигация включена. Используйте кнопки снизу для перехода между разделами."
}

pub fn main_menu() -> &'static str {
    "Главное меню"
}

pub fn not_implemented(section: &str) -> String {
    format!("{section}\n\nЭтот раздел будет добавлен на следующих этапах.")
}
