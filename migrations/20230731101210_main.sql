CREATE TABLE user (
  uuid VARCHAR(36) DEFAULT (UUID()) PRIMARY KEY, 
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  password VARCHAR(255) NOT NULL
);

CREATE TABLE task (
  uuid VARCHAR(36) DEFAULT (UUID()) PRIMARY KEY, 
  name VARCHAR(255) NOT NULL,
  description VARCHAR(255) NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  user_uuid VARCHAR(36) NOT NULL,
  position INT NOT NULL DEFAULT 0,

  KEY user_uuid_idx (user_uuid)
);

INSERT INTO user (name, email, password) VALUES 
('John Doe', 'john@gmail.com', '1234');

INSERT INTO task (name, description, completed, user_uuid) VALUES 
('Task 1', 'Task 1 description', false, (SELECT uuid FROM user WHERE name = 'John Doe'));
