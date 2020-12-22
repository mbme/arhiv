# What is arhiv
* personal database
* tool to manage (create/edit/query/search...), sync, backup documents (including large media files)
* universal, for any kind of information
* local first
* holistic
* resilient
* data format
* cross-linking

# Why
* platform to integrate all the data
* decreasing complexity
* controlling data
* as simple as possible, but not simpler

# Characteristics
* single-user database
* stores documents and attachments
* attachments are immutable
* attachments have BLOBs
* database has revisions 
  - empty database has revision 0
  - primary increases revision on each update

# Prime
* the main instance with all the data (even big attachments like movies)
* there might be only one primary instance at a time
* primary receive data from replicas in a form of Changesets
* primary manages revision 
* allows to rename attachments (by increasing their revision)

# Replica
* local instance which gets its data from the primary
* replica has a full copy of documents and attachments (without BLOBs)
* replica choose which BLOBs to fetch from primary
* replica allows to create/update/archive/erase documents and create attachments
  archive vs erase:
    archived documents are hidden from search by default 
    Prime keeps history of archived documents
    archived documents can be unarchived
    archived documents cannot be updated unless they are unarchived
    erased documents are arhived documents with null data
    Prime removes history of erased documents
    erased documents cannot be unarchived
* replica runs compaction when there are 0 locks

# Sync protocol
* replica sends Changeset
  - baseRev - replica rev
  - documents[] - new or updated documents
  - attachments[] - new attachments
* primary returns ChangesetResponse

# Versioning
* prime keeps previous versions of documents
  - prime saves previous version of a document when a new version arrives
  - prime allows to query previous versions of the document
  - replica doesn't keep previous versions of documents
* prime doesn't keep previous versions of attachments or their metadata

# Backups
* daily, weekly, monthly (oldest backup is one month old)
* rsync incremental backup over local network on usb hdd
* replicas would be on a different devices in different places, so they are kind of "backup" backups
* run integrity checks of all snapshots after backup
* restore strategy
  - an option to restore from any of the incremental backups
  - an option to save replica on the file system
* after restore, apply schema migration if needed

# Document types
* note
* task
* log
* events/calendar/reminders
  - put events into logs
* track
* movie
* photo
* picture
* book
* game (+ blobs)
* software
* bookmark
* snippet
* inventory item
* list
  - acts as a playlist for tracks/movies, topic for tasks
