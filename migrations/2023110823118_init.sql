-- Create users table.
create table if not exists users
(
    id           serial primary key,
    username     text not null unique,
    access_token text not null
);