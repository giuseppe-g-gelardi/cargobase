use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Comparison operators for query conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// Equal to (=)
    Eq,
    /// Not equal to (!=)
    Ne,
    /// Greater than (>)
    Gt,
    /// Greater than or equal to (>=)
    Gte,
    /// Less than (<)
    Lt,
    /// Less than or equal to (<=)
    Lte,
    /// Pattern matching (LIKE)
    Like,
    /// In array
    In,
    /// Not in array
    NotIn,
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOp::Eq => write!(f, "="),
            ComparisonOp::Ne => write!(f, "!="),
            ComparisonOp::Gt => write!(f, ">"),
            ComparisonOp::Gte => write!(f, ">="),
            ComparisonOp::Lt => write!(f, "<"),
            ComparisonOp::Lte => write!(f, "<="),
            ComparisonOp::Like => write!(f, "LIKE"),
            ComparisonOp::In => write!(f, "IN"),
            ComparisonOp::NotIn => write!(f, "NOT IN"),
        }
    }
}

/// Sort order for ORDER BY clauses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

/// Logical operators for combining conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOp {
    /// AND condition
    And,
    /// OR condition
    Or,
}

impl fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOp::And => write!(f, "AND"),
            LogicalOp::Or => write!(f, "OR"),
        }
    }
}

/// A single query condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ComparisonOp,
    pub value: Value,
}

impl Condition {
    pub fn new(field: impl Into<String>, operator: ComparisonOp, value: Value) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    /// Evaluate this condition against a row's data
    pub fn evaluate(&self, row_data: &Value) -> bool {
        let field_value = match row_data.get(&self.field) {
            Some(v) => v,
            None => return false,
        };

        match self.operator {
            ComparisonOp::Eq => field_value == &self.value,
            ComparisonOp::Ne => field_value != &self.value,
            ComparisonOp::Gt => Self::compare_gt(field_value, &self.value),
            ComparisonOp::Gte => Self::compare_gte(field_value, &self.value),
            ComparisonOp::Lt => Self::compare_lt(field_value, &self.value),
            ComparisonOp::Lte => Self::compare_lte(field_value, &self.value),
            ComparisonOp::Like => Self::compare_like(field_value, &self.value),
            ComparisonOp::In => Self::compare_in(field_value, &self.value),
            ComparisonOp::NotIn => !Self::compare_in(field_value, &self.value),
        }
    }

    fn compare_gt(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a > b
                } else {
                    false
                }
            }
            (Value::String(a), Value::String(b)) => a > b,
            _ => false,
        }
    }

    fn compare_gte(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a >= b
                } else {
                    false
                }
            }
            (Value::String(a), Value::String(b)) => a >= b,
            _ => false,
        }
    }

    fn compare_lt(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a < b
                } else {
                    false
                }
            }
            (Value::String(a), Value::String(b)) => a < b,
            _ => false,
        }
    }

    fn compare_lte(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a <= b
                } else {
                    false
                }
            }
            (Value::String(a), Value::String(b)) => a <= b,
            _ => false,
        }
    }

    fn compare_like(a: &Value, pattern: &Value) -> bool {
        if let (Some(a_str), Some(pattern_str)) = (a.as_str(), pattern.as_str()) {
            // Simple pattern matching: % = wildcard
            let regex_pattern = pattern_str
                .replace('%', ".*")
                .replace('_', ".");
            
            if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                return re.is_match(a_str);
            }
        }
        false
    }

    fn compare_in(value: &Value, array: &Value) -> bool {
        if let Value::Array(arr) = array {
            arr.contains(value)
        } else {
            false
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format value for display
        let value_str = match &self.value {
            Value::String(s) => format!("'{}'", s),
            Value::Array(arr) => format!("({})", 
                arr.iter()
                    .map(|v| match v {
                        Value::String(s) => format!("'{}'", s),
                        other => other.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            other => other.to_string(),
        };
        
        write!(f, "{} {} {}", self.field, self.operator, value_str)
    }
}

/// A group of conditions combined with a logical operator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionGroup {
    pub conditions: Vec<Condition>,
    pub operator: LogicalOp,
}

impl ConditionGroup {
    pub fn new(operator: LogicalOp) -> Self {
        Self {
            conditions: Vec::new(),
            operator,
        }
    }

    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }

    /// Evaluate all conditions in this group
    pub fn evaluate(&self, row_data: &Value) -> bool {
        if self.conditions.is_empty() {
            return true;
        }

        match self.operator {
            LogicalOp::And => self.conditions.iter().all(|c| c.evaluate(row_data)),
            LogicalOp::Or => self.conditions.iter().any(|c| c.evaluate(row_data)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eq_operator() {
        let condition = Condition::new("age", ComparisonOp::Eq, json!(25));
        let row = json!({"age": 25, "name": "John"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 30, "name": "Jane"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_ne_operator() {
        let condition = Condition::new("age", ComparisonOp::Ne, json!(25));
        let row = json!({"age": 30, "name": "Jane"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 25, "name": "John"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_gt_operator() {
        let condition = Condition::new("age", ComparisonOp::Gt, json!(25));
        let row = json!({"age": 30, "name": "Jane"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 20, "name": "Bob"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_gte_operator() {
        let condition = Condition::new("age", ComparisonOp::Gte, json!(25));
        let row = json!({"age": 25, "name": "John"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 30, "name": "Jane"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 20, "name": "Bob"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_lt_operator() {
        let condition = Condition::new("age", ComparisonOp::Lt, json!(25));
        let row = json!({"age": 20, "name": "Bob"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 30, "name": "Jane"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_lte_operator() {
        let condition = Condition::new("age", ComparisonOp::Lte, json!(25));
        let row = json!({"age": 25, "name": "John"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 20, "name": "Bob"});
        assert!(condition.evaluate(&row));

        let row = json!({"age": 30, "name": "Jane"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_in_operator() {
        let condition = Condition::new("status", ComparisonOp::In, json!(["active", "pending"]));
        let row = json!({"status": "active", "name": "John"});
        assert!(condition.evaluate(&row));

        let row = json!({"status": "inactive", "name": "Jane"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_not_in_operator() {
        let condition = Condition::new("status", ComparisonOp::NotIn, json!(["active", "pending"]));
        let row = json!({"status": "inactive", "name": "Jane"});
        assert!(condition.evaluate(&row));

        let row = json!({"status": "active", "name": "John"});
        assert!(!condition.evaluate(&row));
    }

    #[test]
    fn test_and_condition_group() {
        let mut group = ConditionGroup::new(LogicalOp::And);
        group.add_condition(Condition::new("age", ComparisonOp::Gt, json!(25)));
        group.add_condition(Condition::new("status", ComparisonOp::Eq, json!("active")));

        let row = json!({"age": 30, "status": "active"});
        assert!(group.evaluate(&row));

        let row = json!({"age": 30, "status": "inactive"});
        assert!(!group.evaluate(&row));

        let row = json!({"age": 20, "status": "active"});
        assert!(!group.evaluate(&row));
    }

    #[test]
    fn test_or_condition_group() {
        let mut group = ConditionGroup::new(LogicalOp::Or);
        group.add_condition(Condition::new("age", ComparisonOp::Lt, json!(18)));
        group.add_condition(Condition::new("age", ComparisonOp::Gt, json!(65)));

        let row = json!({"age": 16});
        assert!(group.evaluate(&row));

        let row = json!({"age": 70});
        assert!(group.evaluate(&row));

        let row = json!({"age": 30});
        assert!(!group.evaluate(&row));
    }
}
