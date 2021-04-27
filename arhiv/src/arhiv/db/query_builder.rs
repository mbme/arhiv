use std::cell::RefCell;

pub struct QueryBuilder {
    what: Vec<String>,
    from: String,
    where_conditions: Vec<String>,
    order_by_conditions: Vec<String>,
    limit: Option<i32>,
    offset: Option<u32>,

    params: RefCell<Vec<String>>,
}

impl QueryBuilder {
    pub fn select(what: &str, from: &str) -> Self {
        QueryBuilder {
            what: vec![what.to_string()],
            from: from.to_string(),
            where_conditions: vec![],
            order_by_conditions: vec![],
            limit: None,
            offset: None,

            params: RefCell::new(vec![]),
        }
    }

    pub fn and_select<S: Into<String>>(&mut self, what: S) {
        self.what.push(what.into());
    }

    // returns param name
    pub fn param<S: Into<String>>(&self, param: S) -> String {
        self.params.borrow_mut().push(param.into());

        // this works because in rusqlite, param numeration starts from 1
        format!("?{}", self.params.borrow().len())
    }

    pub fn where_condition<S: Into<String>>(&mut self, condition: S) {
        self.where_conditions.push(condition.into());
    }

    pub fn order_by<S: Into<String>>(&mut self, condition: S, asc: bool) {
        self.order_by_conditions.push(format!(
            "{} {}",
            condition.into(),
            if asc { "ASC" } else { "DESC" }
        ));
    }

    pub fn limit(&mut self, limit: i32) {
        self.limit = Some(limit);
    }

    pub fn offset(&mut self, offset: u32) {
        self.offset = Some(offset);
    }

    pub fn build(self) -> (String, Vec<String>) {
        let mut query = format!("SELECT {} FROM {}", self.what.join(", "), self.from);

        if !self.where_conditions.is_empty() {
            query += " WHERE ";
            query += &self.where_conditions.join(" AND ");
        }

        if !self.order_by_conditions.is_empty() {
            query += " ORDER BY ";
            query += &self.order_by_conditions.join(", ");
        }

        if let Some(limit) = self.limit {
            query += &format!(" LIMIT {}", limit);
        }

        if let Some(offset) = self.offset {
            query += &format!(" OFFSET {}", offset);
        }

        (query, self.params.into_inner())
    }
}
