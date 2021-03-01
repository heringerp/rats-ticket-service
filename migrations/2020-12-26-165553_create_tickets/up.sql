CREATE TABLE tickets (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	description VARCHAR NOT NULL,
	approved BOOLEAN NOT NULL DEFAULT 0,
	approver INTEGER NOT NULL, 
	requestor INTEGER NOT NULL,
    filename VARCHAR NOT NULL,
	FOREIGN KEY(approver) REFERENCES accounts(id),
	FOREIGN KEY(requestor) REFERENCES accounts(id)
);

INSERT INTO tickets (description, approver, requestor, filename) VALUES ("demo task", 1, 0, "ba.doc");
INSERT INTO tickets (description, approver, requestor, filename) VALUES ("demo task 2", 1, 0, "ba.doc");
