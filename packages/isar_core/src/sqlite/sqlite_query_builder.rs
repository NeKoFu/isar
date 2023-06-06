use std::vec;

use super::sql::filter_sql;
use super::sqlite_collection::SQLiteCollection;
use super::sqlite_query::SQLiteQuery;
use crate::core::filter::Filter;
use crate::core::query_builder::{IsarQueryBuilder, Sort};
use itertools::Itertools;

pub struct SQLiteQueryBuilder<'a> {
    pub(crate) collection: &'a SQLiteCollection,
    collection_index: u16,
    filter: Option<Filter>,
    sort: Vec<(&'a str, Sort, bool)>,
    distinct: Vec<(&'a str, bool)>,
}

impl SQLiteQueryBuilder<'_> {
    pub fn new<'a>(
        collection: &'a SQLiteCollection,
        collection_index: u16,
    ) -> SQLiteQueryBuilder<'a> {
        SQLiteQueryBuilder {
            collection,
            collection_index,
            filter: None,
            sort: Vec::new(),
            distinct: Vec::new(),
        }
    }
}

impl<'a> IsarQueryBuilder for SQLiteQueryBuilder<'a> {
    type Query = SQLiteQuery;

    fn set_filter(&mut self, filter: Filter) {
        self.filter = Some(filter);
    }

    fn add_sort(&mut self, property_index: u16, sort: Sort, case_sensitive: bool) {
        self.sort.push((
            self.collection.get_property_name(property_index),
            sort,
            case_sensitive,
        ));
    }

    fn add_distinct(&mut self, property_index: u16, case_sensitive: bool) {
        self.distinct.push((
            self.collection.get_property_name(property_index),
            case_sensitive,
        ));
    }

    fn build(self) -> Self::Query {
        let mut filter_values = vec![];

        let mut sql = String::new();
        sql.push_str("FROM ");
        sql.push_str(&self.collection.name);
        if let Some(filter) = self.filter {
            sql.push_str(" WHERE ");
            let (filter_sql, values) = filter_sql(self.collection, filter);
            sql.push_str(&filter_sql);
            filter_values = values;
        }
        if !self.sort.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(
                &self
                    .sort
                    .iter()
                    .map(|(prop, sort, case_sensitive)| match sort {
                        Sort::Asc => format!(
                            "{} COLLATE {}",
                            prop,
                            if *case_sensitive { "NOCASE" } else { "BINARY" }
                        ),
                        Sort::Desc => format!(
                            "{} COLLATE {} DESC",
                            prop,
                            if *case_sensitive { "NOCASE" } else { "BINARY" }
                        ),
                    })
                    .join(","),
            );
        }
        if !self.distinct.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(
                &self
                    .distinct
                    .iter()
                    .map(|(prop, case_sensitive)| {
                        format!(
                            "{} COLLATE {}",
                            prop,
                            if *case_sensitive { "NOCASE" } else { "BINARY" }
                        )
                    })
                    .join(","),
            );
        }

        let has_sort_distinct = !self.sort.is_empty() || !self.distinct.is_empty();
        SQLiteQuery::new(self.collection_index, sql, has_sort_distinct, filter_values)
    }
}
