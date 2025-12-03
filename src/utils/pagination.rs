use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        (self.page - 1).max(0) * self.per_page
    }

    pub fn limit(&self) -> i64 {
        self.per_page.clamp(1, 100)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub total_items: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl Pagination {
    #[allow(dead_code)]
    pub fn new(page: i64, per_page: i64, total_items: i64) -> Self {
        let total_pages = (total_items as f64 / per_page as f64).ceil() as i64;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Pagination {
            page,
            per_page,
            total_pages,
            total_items,
            has_next,
            has_prev,
        }
    }

    #[allow(dead_code)]
    pub fn next_page(&self) -> Option<i64> {
        if self.has_next {
            Some(self.page + 1)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn prev_page(&self) -> Option<i64> {
        if self.has_prev {
            Some(self.page - 1)
        } else {
            None
        }
    }
}
