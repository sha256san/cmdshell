pub const CREATE_HISTORY_TABLE: &str = "
CREATE TABLE IF NOT EXISTS command_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    cwd TEXT,
    exit_code INTEGER,
    executed_at INTEGER NOT NULL,
    execution_count INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_history_command ON command_history(command);
CREATE INDEX IF NOT EXISTS idx_history_executed_at ON command_history(executed_at DESC);
CREATE INDEX IF NOT EXISTS idx_history_count ON command_history(execution_count DESC);
";
