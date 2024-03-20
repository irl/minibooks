/* During early development, expect this to change often and with no migrations, just edits to this initial schema. */

DROP TABLE IF EXISTS account;
DROP TABLE IF EXISTS batch;
DROP TABLE IF EXISTS journal;
DROP TABLE IF EXISTS entry;
DROP TABLE IF EXISTS bank_statement_entry;
DROP TABLE IF EXISTS settings;

CREATE TABLE account
(
    id   INTEGER PRIMARY KEY,
    name TEXT(140) NOT NULL,
    type TEXT NOT NULL,
    archived BOOLEAN DEFAULT FALSE,
    confidential BOOLEAN DEFAULT FALSE
);

INSERT INTO account (id, name, type)
VALUES (100, 'Cash', 'cash');

CREATE TABLE batch
(
    id   INTEGER PRIMARY KEY,
    date DATE NOT NULL
);

CREATE TABLE journal
(
    id                     INTEGER PRIMARY KEY,
    batch_id               INTEGER,
    unstructured_narrative TEXT(140)
);

CREATE TABLE entry
(
    id         INTEGER PRIMARY KEY,
    journal_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    amount     INTEGER NOT NULL
);

CREATE TABLE bank_statement_entry
(
    id                     INTEGER PRIMARY KEY,
    account                INTEGER NOT NULL,
    amt                    INTEGER NOT NULL,
    unstructured_narrative TEXT(140)
);

CREATE TABLE settings
(
    name     TEXT PRIMARY KEY,
    intValue INTEGER,
    strValue TEXT
);

/* Asset Accounts 100-199 */
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountCash', 101);
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountCurrentAsset', 120);
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountNonCurrentAsset', 180);

/* Liability Accounts 200-299 */
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountCurrentLiability', 200);
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountNonCurrentLiability', 280);

/* Equity Accounts 300-399 */
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountEquity', 300);

/* Revenue Accounts 400-499 */
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountRevenue', 400);
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountOtherIncome', 480);

/* Expense Accounts 500-599 */
INSERT OR REPLACE INTO settings (name, intValue) VALUES ('nextAccountExpense', 500);
