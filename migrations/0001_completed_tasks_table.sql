CREATE TYPE "difficulty" AS ENUM ('TRIVIAL', 'EASY', 'MEDIUM', 'HARD');

CREATE TYPE "subtask" AS ("text" TEXT, "completed" BOOLEAN);

CREATE TABLE IF NOT EXISTS completed_task (
    "id" UUID PRIMARY KEY,
    "text" TEXT,
    "task_type" VARCHAR(14) NOT NULL,
    "difficulty" "difficulty" NOT NULL,
    "notes" VARCHAR(10),
    "date" TIMESTAMPTZ,
    "completed_at" TIMESTAMPTZ,
    "checklist" "subtask" []
);
