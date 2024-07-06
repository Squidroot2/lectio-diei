-- Add migration script here
CREATE TABLE IF NOT EXISTS reading (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lectionary_id TEXT NOT NULL,
    reading_type TEXT NOT NULL,
    location TEXT NOT NULL,
    content TEXT NOT NULL,
    FOREIGN KEY (lectionary_id) REFERENCES lectionary(id) ON DELETE CASCADE,
    CHECK(reading_type IN ('first_reading', 'second_reading', 'psalm', 'gospel'))
);
