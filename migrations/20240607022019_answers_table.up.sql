-- Add up migration script here
create table if not exists answers (
    id serial primary key,
    content text not null,
    created_on timestamp not null default now(),
    corresponding_question integer references questions
);