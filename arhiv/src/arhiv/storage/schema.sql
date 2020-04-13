BEGIN;

PRAGMA foreign_keys = ON;

create table documents (
  id text not null,
  rev number,
  created_at text not null,
  updated_at text not null,
  archived boolean not null,

  type text not null,
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

create table documents_documents (
  document_id text not null,
  other_document_id text not null,
  rev number,

  primary key (document_id, other_document_id, rev),
  foreign key (document_id) references documents(id) on delete restrict,
  foreign key (other_document_id) references documents(id) on delete restrict
);

create table documents_attachments (
  document_id text not null,
  attachment_id text not null,
  rev number,

  primary key (document_id, attachment_id, rev),
  foreign key (document_id) references documents(id) on delete restrict,
  foreign key (attachment_id) references attachments(id) on delete restrict
);

COMMIT;
