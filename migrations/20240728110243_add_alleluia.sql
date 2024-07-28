-- Create new table to change reading_type constraint
CREATE TABLE IF NOT EXISTS new_reading (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lectionary_id TEXT NOT NULL,
    reading_type TEXT NOT NULL,
    location TEXT NOT NULL,
    content TEXT NOT NULL,
    FOREIGN KEY (lectionary_id) REFERENCES lectionary(id) ON DELETE CASCADE,
    CHECK(reading_type IN ('first_reading', 'second_reading', 'psalm', 'gospel', 'alleluia'))
);
INSERT INTO new_reading SELECT * FROM reading;
DROP TABLE reading;
ALTER TABLE new_reading RENAME TO reading;

-- Removes all lectionaries with no alleluia (anything pre-0.3)
DELETE FROM lectionary WHERE NOT EXISTS (
    SELECT * FROM reading
        WHERE reading_type == 'alleluia' AND lectionary_id == lectionary.id
);

