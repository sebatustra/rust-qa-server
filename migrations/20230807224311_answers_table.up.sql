CREATE TABLE IF NOT EXISTS answers (
 id serial PRIMARY KEY,
 content TEXT NOT NULL,
 created_on TIMESTAMP NOT NULL DEFAULT NOW(),
 question_id integer REFERENCES questions
);
