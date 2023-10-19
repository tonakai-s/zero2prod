-- Create Subscriptions Table
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
);