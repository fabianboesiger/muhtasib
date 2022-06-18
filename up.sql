sudo su - postgres
psql -d bazaar

create database bazaar;
create user bazaar with encrypted password 'sultan';
create user muhtasib with encrypted password 'sultan';
grant all privileges on database bazaar to bazaar;
grant all privileges on database bazaar to muhtasib;
\c bazaar
grant all privileges on sessions, equities, orders to bazaar;
grant all privileges on sessions, equities, orders to muhtasib;


DROP TABLE orders;
DROP TABLE equities;
DROP TABLE sessions;
DROP TYPE side;

CREATE TABLE sessions (
    session_id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    exchange TEXT NOT NULL,
    live_trading BOOLEAN NOT NULL,
    abort_reason TEXT,
    create_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE equities (
    session_id UUID REFERENCES sessions(session_id) ON DELETE CASCADE,
    total DECIMAL NOT NULL,
    time TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY(session_id, time)
);

CREATE TYPE side AS ENUM ('BUY', 'SELL');

CREATE TABLE orders (
    order_id UUID PRIMARY KEY,
    session_id UUID REFERENCES sessions(session_id) ON DELETE CASCADE,
    market TEXT NOT NULL,
    side side NOT NULL,
    ordered_size DECIMAL NOT NULL,
    ordered_price DECIMAL NOT NULL,
    ordered_time TIMESTAMP WITH TIME ZONE NOT NULL,
    executed_size DECIMAL,
    executed_price DECIMAL,
    executed_time TIMESTAMP WITH TIME ZONE
);

