CREATE TABLE board (
  uuid VARCHAR(36) DEFAULT (UUID()) PRIMARY KEY, 
  name VARCHAR(255) NOT NULL,
  user_uuid VARCHAR(36) NOT NULL,

  KEY user_uuid_idx (user_uuid) 
);

ALTER TABLE task
ADD COLUMN board_uuid VARCHAR(36) NOT NULL DEFAULT '',
ADD KEY board_uuid_idx (board_uuid);
