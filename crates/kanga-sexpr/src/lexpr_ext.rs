use {
    crate::ParseError,
    lexpr::{Cons, Value},
};

#[allow(unused)]
pub trait LexprExt {
    /// Assert that [`self`] is a boolean value. If it is, return it as a Rust `bool`.
    ///
    /// KiCad uses the symbols `yes` and `no` for booleans.
    fn expect_bool(&self) -> Result<bool, ParseError>;

    /// Assert that [`self`] is a cons cell. If it is, return it as a [`Cons`] cell.
    fn expect_cons(&self) -> Result<&Cons, ParseError>;

    /// Assert that [`self`] is a cons cell with an integer head. If it is, return the head and
    /// the cdr.
    fn expect_cons_with_any_i64_head(&self) -> Result<(i64, &Value), ParseError>;

    /// Assert that [`self`] is a cons cell with a float head. If it is, return the head and
    /// the cdr.
    fn expect_cons_with_any_f64_head(&self) -> Result<(f64, &Value), ParseError>;

    /// Assert that [`self`] is a cons cell with a string head. If it is, return the head and
    /// the cdr.
    fn expect_cons_with_any_str_head(&self) -> Result<(&str, &Value), ParseError>;

    /// Assert that [`self`] is a cons cell with a symbol head. If it is, return the head and
    /// the cdr.
    fn expect_cons_with_any_symbol_head(&self) -> Result<(&str, &Value), ParseError>;

    /// Assert that [`self`] is a cons cell with a particular symbol as its head. If it is, return
    /// the cdr.
    fn expect_cons_with_symbol_head(&self, symbol: &str) -> Result<&Value, ParseError>;

    /// Assert that [`self`] is null (end-of-list).
    fn expect_null(&self) -> Result<(), ParseError>;

    /// Assert that [`self`] is the specified symbol.
    fn expect_symbol(&self, symbol: &str) -> Result<(), ParseError>;
}

impl LexprExt for Cons {
    fn expect_bool(&self) -> Result<bool, ParseError> {
        Err(ParseError::Unexpected(Value::Cons(self.clone())))
    }

    fn expect_cons(&self) -> Result<&Cons, ParseError> {
        Ok(self)
    }

    fn expect_cons_with_symbol_head(&self, symbol: &str) -> Result<&Value, ParseError> {
        let (sym, cdr) = self.expect_cons_with_any_symbol_head()?;
        if sym != symbol {
            Err(ParseError::ExpectedSymbol(Value::Cons(self.clone()), symbol.to_string()))
        } else {
            Ok(cdr)
        }
    }

    fn expect_null(&self) -> Result<(), ParseError> {
        Err(ParseError::ExpectedNil(Value::Cons(self.clone())))
    }

    fn expect_symbol(&self, symbol: &str) -> Result<(), ParseError> {
        Err(ParseError::ExpectedSymbol(Value::Cons(self.clone()), symbol.to_string()))
    }

    fn expect_cons_with_any_i64_head(&self) -> Result<(i64, &Value), ParseError> {
        let car = self.car();
        let cdr = self.cdr();
        let num = car.as_number().ok_or_else(|| ParseError::ExpectedListIntHead(Value::Cons(self.clone())))?;
        let num = num.as_i64().ok_or_else(|| ParseError::ExpectedListIntHead(Value::Cons(self.clone())))?;
        Ok((num, cdr))
    }

    fn expect_cons_with_any_f64_head(&self) -> Result<(f64, &Value), ParseError> {
        let car = self.car();
        let cdr = self.cdr();
        let num = car.as_number().ok_or_else(|| ParseError::ExpectedListFloatHead(Value::Cons(self.clone())))?;
        let num = num.as_f64().ok_or_else(|| ParseError::ExpectedListFloatHead(Value::Cons(self.clone())))?;
        Ok((num, cdr))
    }

    fn expect_cons_with_any_str_head(&self) -> Result<(&str, &Value), ParseError> {
        let car = self.car();
        let cdr = self.cdr();
        let s = car.as_str().ok_or_else(|| ParseError::ExpectedListStrHead(Value::Cons(self.clone())))?;
        Ok((s, cdr))
    }

    fn expect_cons_with_any_symbol_head(&self) -> Result<(&str, &Value), ParseError> {
        let car = self.car();
        let cdr = self.cdr();
        let sym = car.as_symbol().ok_or_else(|| ParseError::ExpectedListSymbolHead(Value::Cons(self.clone())))?;
        Ok((sym, cdr))
    }
}

impl LexprExt for Value {
    fn expect_bool(&self) -> Result<bool, ParseError> {
        if let Some(sym) = self.as_symbol() {
            match sym {
                "yes" | "y" | "true" | "t" => Ok(true),
                "no" | "n" | "false" | "f" => Ok(false),
                _ => Err(ParseError::Unexpected(self.clone())),
            }
        } else if let Some(value) = self.as_bool() {
            Ok(value)
        } else if matches!(self, Value::Nil | Value::Null) {
            Ok(false)
        } else {
            Err(ParseError::Unexpected(self.clone()))
        }
    }

    fn expect_cons(&self) -> Result<&Cons, ParseError> {
        self.as_cons().ok_or_else(|| ParseError::ExpectedList(self.clone()))
    }

    fn expect_cons_with_symbol_head(&self, symbol: &str) -> Result<&Value, ParseError> {
        let (sym, cdr) = self.expect_cons_with_any_symbol_head()?;
        if sym != symbol {
            Err(ParseError::ExpectedSymbol(self.clone(), symbol.to_string()))
        } else {
            Ok(cdr)
        }
    }

    fn expect_null(&self) -> Result<(), ParseError> {
        if !self.is_null() {
            Err(ParseError::ExpectedNil(self.clone()))
        } else {
            Ok(())
        }
    }

    fn expect_symbol(&self, symbol: &str) -> Result<(), ParseError> {
        if self.as_symbol() != Some(symbol) {
            Err(ParseError::ExpectedSymbol(self.clone(), symbol.to_string()))
        } else {
            Ok(())
        }
    }

    fn expect_cons_with_any_i64_head(&self) -> Result<(i64, &Value), ParseError> {
        let cons = self.expect_cons()?;
        let car = cons.car();
        let cdr = cons.cdr();
        let num = car.as_number().ok_or_else(|| ParseError::ExpectedListIntHead(self.clone()))?;
        let num = num.as_i64().ok_or_else(|| ParseError::ExpectedListIntHead(self.clone()))?;
        Ok((num, cdr))
    }

    fn expect_cons_with_any_f64_head(&self) -> Result<(f64, &Value), ParseError> {
        let cons = self.expect_cons()?;
        let car = cons.car();
        let cdr = cons.cdr();
        let num = car.as_number().ok_or_else(|| ParseError::ExpectedListFloatHead(self.clone()))?;
        let num = num.as_f64().ok_or_else(|| ParseError::ExpectedListFloatHead(self.clone()))?;
        Ok((num, cdr))
    }

    fn expect_cons_with_any_str_head(self: &Value) -> Result<(&str, &Value), ParseError> {
        let cons = self.expect_cons()?;
        let car = cons.car();
        let cdr = cons.cdr();
        let s = car.as_str().ok_or_else(|| ParseError::ExpectedListStrHead(self.clone()))?;
        Ok((s, cdr))
    }

    fn expect_cons_with_any_symbol_head(&self) -> Result<(&str, &Value), ParseError> {
        let cons = self.expect_cons()?;
        let car = cons.car();
        let cdr = cons.cdr();
        let sym = car.as_symbol().ok_or_else(|| ParseError::ExpectedListSymbolHead(self.clone()))?;
        Ok((sym, cdr))
    }
}
