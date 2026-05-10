//! Деньги как доменный value object.
//!
//! В бизнес-коде деньги нельзя хранить как `f32`/`f64`: двоичная арифметика с
//! плавающей точкой не гарантирует точное представление десятичных дробей, а
//! для цен, оплат и итогов даже копеечная ошибка недопустима. Поэтому модуль
//! хранит сумму в минимальных единицах валюты:
//! - BYN хранится в копейках;
//! - USD хранится в центах.
//!
//! Например, `10.50 BYN` внутри будет `1050`, а не `10.50`. Такой формат
//! упрощает сравнение, сложение, вычитание и сохранение в базе данных.
//!
//! Алгоритм работы с деньгами здесь строится вокруг трех инвариантов:
//! 1. Сумма не может быть отрицательной.
//! 2. Арифметика разрешена только внутри одной валюты.
//! 3. Переполнение `i64` не маскируется, а возвращается как ошибка.
//!
//! Благодаря этому прикладной слой получает предсказуемую модель денег и не
//! размазывает проверки по сервисам, обработчикам и инфраструктуре.

use thiserror::Error;

/// Поддерживаемая валюта.
///
/// Основная рабочая валюта проекта - BYN.
/// USD нужен для отображения статистики и отчетов.
///
/// Валюта является частью значения `Money`, а не внешней метаданной. Это
/// принципиально: `100 BYN` и `100 USD` - разные деньги, даже если числовая
/// часть совпадает.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    /// Белорусский рубль. Минимальная единица хранения - копейка.
    Byn,
    /// Доллар США. Минимальная единица хранения - цент.
    Usd,
}

/// Выводит ISO-подобный код валюты для UI, логов и сериализуемых сообщений.
///
/// Здесь нет локализации и бизнес-форматирования: тип отвечает только за
/// стабильное короткое представление валюты.
impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::Byn => write!(f, "BYN"),
            Currency::Usd => write!(f, "USD"),
        }
    }
}

/// Денежная сумма в минимальных единицах валюты.
///
/// Для BYN это копейки.
/// Для USD это центы.
///
/// Например:
/// - `10.50 BYN` хранится как `1050`
/// - `25.99 USD` хранится как `2599`
///
/// Поля закрыты намеренно. Создать `Money` можно только через конструкторы,
/// которые проверяют инварианты. Это защищает доменную модель от случайных
/// отрицательных сумм и от ручной сборки неконсистентных значений.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Money {
    /// Числовое значение в минимальных единицах валюты.
    ///
    /// Для отображения это значение делится на 100, но для арифметики оно
    /// остается целым числом. Так мы избегаем ошибок округления.
    amount_minor: i64,
    /// Валюта, в которой выражена сумма.
    currency: Currency,
}

impl Money {
    /// Создает денежную сумму из минимальных единиц.
    ///
    /// Отрицательные суммы запрещены для базовой модели денег.
    ///
    /// Алгоритм конструктора простой, но важный:
    /// 1. Проверяем знак суммы до создания значения.
    /// 2. Если сумма отрицательная, возвращаем доменную ошибку.
    /// 3. Если сумма корректна, сохраняем ее вместе с валютой.
    ///
    /// Возврат `Result` лучше паники: неверная сумма может прийти из формы, API
    /// или базы, и прикладной слой должен иметь возможность обработать ошибку.
    pub fn new(amount_minor: i64, currency: Currency) -> Result<Self, MoneyError> {
        if amount_minor < 0 {
            return Err(MoneyError::NegativeAmount);
        }

        Ok(Self {
            amount_minor,
            currency,
        })
    }

    /// Создает сумму в BYN из копеек.
    ///
    /// Это удобный именованный конструктор для основной валюты проекта. Он
    /// делегирует проверку в `new`, чтобы правило запрета отрицательных сумм
    /// было в одном месте.
    pub fn byn_minor(amount_minor: i64) -> Result<Self, MoneyError> {
        Self::new(amount_minor, Currency::Byn)
    }

    /// Создает сумму в USD из центов.
    ///
    /// Как и `byn_minor`, этот метод не дублирует валидацию. Все правила
    /// создания суммы централизованы в `new`.
    pub fn usd_minor(amount_minor: i64) -> Result<Self, MoneyError> {
        Self::new(amount_minor, Currency::Usd)
    }

    /// Создает нулевую сумму в указанной валюте.
    ///
    /// Ноль всегда валиден, поэтому метод возвращает `Self`, а не `Result`.
    /// Это удобно для накопления итогов: можно начать с `Money::zero(currency)`
    /// и дальше добавлять позиции через `checked_add`.
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount_minor: 0,
            currency,
        }
    }

    /// Возвращает сумму в минимальных единицах.
    ///
    /// Метод нужен для сохранения в базе данных, сравнения и передачи в API,
    /// где деньги ожидаются как целое число минимальных единиц.
    pub fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    /// Возвращает валюту суммы.
    ///
    /// `Currency` копируемый и маленький enum, поэтому возвращаем его по
    /// значению без лишних ссылок и lifetime-ов.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Безопасно складывает две суммы одной валюты.
    ///
    /// Алгоритм сложения:
    /// 1. Сначала проверяем валюту. Складывать BYN и USD без курса нельзя:
    ///    это уже валютная конвертация, а не арифметика денег.
    /// 2. Используем `i64::checked_add`, чтобы не получить тихое переполнение.
    /// 3. Пропускаем результат через `new`, сохраняя единый вход для проверки
    ///    инвариантов.
    ///
    /// Даже если сейчас суммы приходят из доверенного источника, checked-подход
    /// дешевле, чем поиск финансовой ошибки после переполнения.
    pub fn checked_add(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;

        let amount_minor = self
            .amount_minor
            .checked_add(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;

        Self::new(amount_minor, self.currency)
    }

    /// Безопасно вычитает одну сумму из другой.
    ///
    /// Результат не может быть отрицательным.
    ///
    /// Алгоритм вычитания повторяет сложение по структуре:
    /// 1. Валюты должны совпадать.
    /// 2. `checked_sub` защищает от арифметического выхода за пределы `i64`.
    /// 3. `new` отсекает отрицательный результат как доменно недопустимый.
    ///
    /// Это значит, что попытка списать больше, чем есть в сумме, вернет
    /// `MoneyError::NegativeAmount`, а не создаст значение `-10.00 BYN`.
    pub fn checked_sub(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;

        let amount_minor = self
            .amount_minor
            .checked_sub(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;

        Self::new(amount_minor, self.currency)
    }

    /// Безопасно умножает сумму на целое неотрицательное количество.
    ///
    /// Метод нужен для строк ремонта и других сценариев, где цена одной единицы
    /// умножается на количество. Множитель выбран как `u32`, потому что складские
    /// количества в домене представлены `PartQuantity`.
    ///
    /// Алгоритм:
    /// 1. Переводим множитель в `i64`. Любой `u32` помещается в `i64`.
    /// 2. Используем `checked_mul`, чтобы переполнение не превратилось в
    ///    некорректную финансовую сумму.
    /// 3. Пропускаем результат через `Money::new`, сохраняя единый вход для
    ///    проверки неотрицательности.
    pub fn checked_mul_u32(self, multiplier: u32) -> Result<Self, MoneyError> {
        let amount_minor = self
            .amount_minor
            .checked_mul(i64::from(multiplier))
            .ok_or(MoneyError::Overflow)?;

        Self::new(amount_minor, self.currency)
    }

    /// Проверяет, что две суммы выражены в одной валюте.
    ///
    /// Метод приватный, потому что это не отдельная операция домена, а общий
    /// защитный шаг для арифметики. Ошибка содержит обе валюты, чтобы лог или UI
    /// могли показать точную причину отказа.
    fn ensure_same_currency(self, other: Self) -> Result<(), MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }

        Ok(())
    }
}

/// Денежная сумма, которая может быть отрицательной.
///
/// `Money` специально запрещает отрицательные значения, потому что цены,
/// оплаты и остатки к оплате в домене должны быть неотрицательными. Но для
/// аналитики ремонта нужен другой тип: фактическая прибыль может быть ниже
/// нуля, если клиент оплатил меньше себестоимости запчастей.
///
/// Поэтому `SignedMoney` используется для расчетных показателей вроде прибыли,
/// убытка и будущих корректировок. Валюта остается частью значения, а вся
/// арифметика по-прежнему требует совпадения валют.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignedMoney {
    /// Сумма в минимальных единицах валюты. Может быть отрицательной.
    amount_minor: i64,
    /// Валюта суммы.
    currency: Currency,
}

impl SignedMoney {
    /// Создает signed-сумму без запрета отрицательных значений.
    ///
    /// В отличие от `Money::new`, этот конструктор не возвращает `Result`:
    /// отрицательные значения являются валидной частью модели.
    pub fn new(amount_minor: i64, currency: Currency) -> Self {
        Self {
            amount_minor,
            currency,
        }
    }

    /// Создает нулевую signed-сумму в указанной валюте.
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount_minor: 0,
            currency,
        }
    }

    /// Возвращает сумму в минимальных единицах валюты.
    pub fn amount_minor(&self) -> i64 {
        self.amount_minor
    }

    /// Возвращает валюту суммы.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Проверяет, что сумма больше нуля.
    pub fn is_positive(&self) -> bool {
        self.amount_minor > 0
    }

    /// Проверяет, что сумма равна нулю.
    pub fn is_zero(&self) -> bool {
        self.amount_minor == 0
    }

    /// Проверяет, что сумма меньше нуля.
    pub fn is_negative(&self) -> bool {
        self.amount_minor < 0
    }

    /// Безопасно складывает две signed-суммы одной валюты.
    ///
    /// Валюты должны совпадать, а переполнение `i64` возвращается как ошибка.
    pub fn checked_add(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;

        let amount_minor = self
            .amount_minor
            .checked_add(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;

        Ok(Self::new(amount_minor, self.currency))
    }

    /// Безопасно вычитает одну signed-сумму из другой.
    ///
    /// В отличие от `Money::checked_sub`, отрицательный результат разрешен.
    pub fn checked_sub(self, other: Self) -> Result<Self, MoneyError> {
        self.ensure_same_currency(other)?;

        let amount_minor = self
            .amount_minor
            .checked_sub(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;

        Ok(Self::new(amount_minor, self.currency))
    }

    /// Проверяет, что две signed-суммы выражены в одной валюте.
    fn ensure_same_currency(self, other: Self) -> Result<(), MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }

        Ok(())
    }
}

/// Переводит обычные неотрицательные деньги в signed-представление.
///
/// Это используется в расчетах прибыли: входные цены и оплаты остаются
/// `Money`, а результат вычисления может стать `SignedMoney`.
impl From<Money> for SignedMoney {
    fn from(value: Money) -> Self {
        Self {
            amount_minor: value.amount_minor(),
            currency: value.currency(),
        }
    }
}

/// Форматирует signed-сумму в человекочитаемый вид `[-]major.minor CURRENCY`.
impl std::fmt::Display for SignedMoney {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = if self.amount_minor < 0 { "-" } else { "" };
        let abs = self.amount_minor.abs();
        let major = abs / 100;
        let minor = abs % 100;

        write!(f, "{}{}.{:02} {}", sign, major, minor, self.currency)
    }
}

/// Форматирует сумму в человекочитаемый вид `major.minor CURRENCY`.
///
/// Алгоритм форматирования:
/// 1. Делим минимальные единицы на 100 и получаем основную часть.
/// 2. Остаток от деления на 100 становится дробной частью.
/// 3. Дробную часть печатаем через `{:02}`, чтобы `10.05` не превратилось в
///    `10.5`.
///
/// Метод предполагает валюты с двумя знаками после запятой. Это соответствует
/// текущим `BYN` и `USD`. Если появится валюта с другим количеством minor units,
/// это место нужно будет расширить вместе с моделью `Currency`.
impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let major = self.amount_minor / 100;
        let minor = self.amount_minor % 100;

        write!(f, "{}.{:02} {}", major, minor, self.currency)
    }
}

/// Ошибка работы с денежными суммами.
///
/// Ошибки отражают именно доменные и арифметические ограничения `Money`.
/// Они не занимаются парсингом пользовательских строк и не описывают ошибки
/// валютных курсов: это ответственность других слоев.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MoneyError {
    /// Сумма меньше нуля. Для текущей модели денег отрицательные значения
    /// запрещены и должны моделироваться отдельным бизнес-понятием, если
    /// когда-нибудь понадобятся долги, возвраты или корректировки.
    #[error("money amount cannot be negative")]
    NegativeAmount,

    /// Попытка сложить или вычесть суммы в разных валютах.
    ///
    /// Автоматически конвертировать валюты здесь нельзя: для этого нужен курс,
    /// дата курса, правила округления и источник данных.
    #[error("currency mismatch: left={left:?}, right={right:?}")]
    CurrencyMismatch { left: Currency, right: Currency },

    /// Арифметическая операция вышла за пределы `i64`.
    ///
    /// Такая ошибка маловероятна для обычных сумм, но checked-арифметика делает
    /// поведение явным и защищает от тихой порчи данных.
    #[error("money amount overflow")]
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::{Currency, Money, MoneyError, SignedMoney};

    /// Базовый конструктор должен сохранять сумму и валюту без скрытых
    /// преобразований: вызывающий код уже передает значение в minor units.
    #[test]
    fn new_creates_money_from_minor_units() {
        let money = Money::new(1050, Currency::Byn).unwrap();

        assert_eq!(money.amount_minor(), 1050);
        assert_eq!(money.currency(), Currency::Byn);
    }

    /// Отрицательная сумма отсекается на входе, чтобы в домене не появлялись
    /// некорректные значения `Money`.
    #[test]
    fn new_rejects_negative_amount() {
        let error = Money::new(-1, Currency::Byn).unwrap_err();

        assert_eq!(error, MoneyError::NegativeAmount);
    }

    /// Именованный конструктор BYN фиксирует валюту и оставляет всю валидацию
    /// общему `new`.
    #[test]
    fn byn_minor_creates_byn_money() {
        let money = Money::byn_minor(2500).unwrap();

        assert_eq!(money.amount_minor(), 2500);
        assert_eq!(money.currency(), Currency::Byn);
    }

    /// Именованный конструктор USD нужен, чтобы в местах вызова не путать валюту
    /// и явно читать намерение бизнес-операции.
    #[test]
    fn usd_minor_creates_usd_money() {
        let money = Money::usd_minor(2599).unwrap();

        assert_eq!(money.amount_minor(), 2599);
        assert_eq!(money.currency(), Currency::Usd);
    }

    /// Ноль является валидным значением в любой поддерживаемой валюте и не
    /// требует `Result`.
    #[test]
    fn zero_creates_zero_amount_in_selected_currency() {
        let money = Money::zero(Currency::Usd);

        assert_eq!(money.amount_minor(), 0);
        assert_eq!(money.currency(), Currency::Usd);
    }

    /// Display для валюты должен быть стабильным: эти строки могут попасть в UI,
    /// логи, отчеты и внешние сообщения.
    #[test]
    fn currency_display_returns_currency_code() {
        assert_eq!(Currency::Byn.to_string(), "BYN");
        assert_eq!(Currency::Usd.to_string(), "USD");
    }

    /// Деньги отображаются через основную и дробную часть. Важно проверять
    /// ведущий ноль у minor part, иначе `10.05` легко превратить в `10.5`.
    #[test]
    fn money_display_formats_major_and_minor_units() {
        let money = Money::byn_minor(1005).unwrap();

        assert_eq!(money.to_string(), "10.05 BYN");
    }

    /// Сложение двух сумм одной валюты должно вернуть ту же валюту и сумму в
    /// minor units без округления.
    #[test]
    fn checked_add_adds_money_with_same_currency() {
        let left = Money::byn_minor(1000).unwrap();
        let right = Money::byn_minor(250).unwrap();

        let result = left.checked_add(right).unwrap();

        assert_eq!(result.amount_minor(), 1250);
        assert_eq!(result.currency(), Currency::Byn);
    }

    /// Сложение разных валют запрещено. Это защищает домен от неявной валютной
    /// конвертации без курса и правил округления.
    #[test]
    fn checked_add_rejects_currency_mismatch() {
        let left = Money::byn_minor(1000).unwrap();
        let right = Money::usd_minor(1000).unwrap();

        let error = left.checked_add(right).unwrap_err();

        assert_eq!(
            error,
            MoneyError::CurrencyMismatch {
                left: Currency::Byn,
                right: Currency::Usd,
            }
        );
    }

    /// `checked_add` должен явно вернуть overflow, если сумма выходит за пределы
    /// `i64`, а не допустить тихое переполнение.
    #[test]
    fn checked_add_rejects_integer_overflow() {
        let left = Money::new(i64::MAX, Currency::Byn).unwrap();
        let right = Money::byn_minor(1).unwrap();

        let error = left.checked_add(right).unwrap_err();

        assert_eq!(error, MoneyError::Overflow);
    }

    /// Вычитание одной валюты работает симметрично сложению: результат остается
    /// в той же валюте и хранится в minor units.
    #[test]
    fn checked_sub_subtracts_money_with_same_currency() {
        let left = Money::byn_minor(1000).unwrap();
        let right = Money::byn_minor(250).unwrap();

        let result = left.checked_sub(right).unwrap();

        assert_eq!(result.amount_minor(), 750);
        assert_eq!(result.currency(), Currency::Byn);
    }

    /// Вычитание разных валют так же запрещено, как и сложение. Арифметика
    /// должна происходить только после явной конвертации в другом слое.
    #[test]
    fn checked_sub_rejects_currency_mismatch() {
        let left = Money::byn_minor(1000).unwrap();
        let right = Money::usd_minor(1000).unwrap();

        let error = left.checked_sub(right).unwrap_err();

        assert_eq!(
            error,
            MoneyError::CurrencyMismatch {
                left: Currency::Byn,
                right: Currency::Usd,
            }
        );
    }

    /// Если вычесть больше, чем есть, арифметически `i64` еще может это
    /// представить, но доменная модель запрещает отрицательные деньги.
    #[test]
    fn checked_sub_rejects_negative_result() {
        let left = Money::byn_minor(100).unwrap();
        let right = Money::byn_minor(250).unwrap();

        let error = left.checked_sub(right).unwrap_err();

        assert_eq!(error, MoneyError::NegativeAmount);
    }

    /// Умножение на ноль должно возвращать нулевую сумму той же валюты.
    #[test]
    fn checked_mul_u32_by_zero_returns_zero() {
        let money = Money::byn_minor(1250).unwrap();

        let result = money.checked_mul_u32(0).unwrap();

        assert_eq!(result, Money::zero(Currency::Byn));
    }

    /// Умножение на единицу не меняет сумму.
    #[test]
    fn checked_mul_u32_by_one_keeps_amount() {
        let money = Money::usd_minor(2599).unwrap();

        let result = money.checked_mul_u32(1).unwrap();

        assert_eq!(result, money);
    }

    /// Умножение на два должно работать как обычная целочисленная арифметика в
    /// minor units без округления.
    #[test]
    fn checked_mul_u32_by_two_doubles_amount() {
        let money = Money::byn_minor(750).unwrap();

        let result = money.checked_mul_u32(2).unwrap();

        assert_eq!(result, Money::byn_minor(1500).unwrap());
    }

    /// Переполнение при умножении возвращается явной ошибкой.
    #[test]
    fn checked_mul_u32_rejects_overflow() {
        let money = Money::new(i64::MAX, Currency::Byn).unwrap();

        let error = money.checked_mul_u32(2).unwrap_err();

        assert_eq!(error, MoneyError::Overflow);
    }

    /// Signed-сумма допускает отрицательные значения и сохраняет валюту.
    #[test]
    fn signed_money_new_allows_negative_amount() {
        let money = SignedMoney::new(-150, Currency::Byn);

        assert_eq!(money.amount_minor(), -150);
        assert_eq!(money.currency(), Currency::Byn);
        assert!(money.is_negative());
        assert!(!money.is_zero());
        assert!(!money.is_positive());
    }

    /// Нулевая signed-сумма нужна как нейтральное значение для расчетов.
    #[test]
    fn signed_money_zero_creates_zero_amount() {
        let money = SignedMoney::zero(Currency::Usd);

        assert_eq!(money.amount_minor(), 0);
        assert_eq!(money.currency(), Currency::Usd);
        assert!(money.is_zero());
    }

    /// Обычные деньги можно безопасно перевести в signed-представление для
    /// расчетов прибыли.
    #[test]
    fn signed_money_from_money_preserves_amount_and_currency() {
        let money = Money::byn_minor(1250).unwrap();

        let signed = SignedMoney::from(money);

        assert_eq!(signed.amount_minor(), 1250);
        assert_eq!(signed.currency(), Currency::Byn);
        assert!(signed.is_positive());
    }

    /// Signed-сложение работает с отрицательными и положительными значениями
    /// без запрета отрицательного результата.
    #[test]
    fn signed_money_checked_add_adds_values() {
        let left = SignedMoney::new(-300, Currency::Byn);
        let right = SignedMoney::new(100, Currency::Byn);

        let result = left.checked_add(right).unwrap();

        assert_eq!(result, SignedMoney::new(-200, Currency::Byn));
    }

    /// Signed-вычитание может вернуть отрицательный результат, что нужно для
    /// расчета убытка.
    #[test]
    fn signed_money_checked_sub_allows_negative_result() {
        let left = SignedMoney::new(100, Currency::Byn);
        let right = SignedMoney::new(350, Currency::Byn);

        let result = left.checked_sub(right).unwrap();

        assert_eq!(result, SignedMoney::new(-250, Currency::Byn));
    }

    /// Арифметика signed-сумм так же запрещает смешивать валюты.
    #[test]
    fn signed_money_checked_add_rejects_currency_mismatch() {
        let left = SignedMoney::new(100, Currency::Byn);
        let right = SignedMoney::new(100, Currency::Usd);

        let error = left.checked_add(right).unwrap_err();

        assert_eq!(
            error,
            MoneyError::CurrencyMismatch {
                left: Currency::Byn,
                right: Currency::Usd,
            }
        );
    }

    /// Переполнение signed-арифметики должно возвращаться как явная ошибка.
    #[test]
    fn signed_money_checked_sub_rejects_integer_overflow() {
        let left = SignedMoney::new(i64::MIN, Currency::Byn);
        let right = SignedMoney::new(1, Currency::Byn);

        let error = left.checked_sub(right).unwrap_err();

        assert_eq!(error, MoneyError::Overflow);
    }

    /// Display должен сохранять знак и ведущий ноль в minor part.
    #[test]
    fn signed_money_display_formats_negative_amount() {
        let money = SignedMoney::new(-1005, Currency::Byn);

        assert_eq!(money.to_string(), "-10.05 BYN");
    }
}
