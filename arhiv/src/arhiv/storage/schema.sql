BEGIN;

create table documents (
  id text not null,
  rev number,
  created_at text not null,
  updated_at text not null,
  archived boolean not null,

  type text not null,
  refs text not null,
  attachment_refs text not null,
  data text not null,

  primary key (id, rev)
);

create table attachments (
  id text not null,
  rev number,
  created_at text not null,
  filename text not null,

  primary key (id, rev)
);

COMMIT;
