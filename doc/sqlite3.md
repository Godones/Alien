# Sqlite3

```
sqlite3
.help
.open test.db
.databases
CREATE TABLE COMPANY(
   ID INT PRIMARY KEY     NOT NULL,
   NAME           TEXT    NOT NULL,
   AGE            INT     NOT NULL,
   ADDRESS        CHAR(50),
   SALARY         REAL
);
.schema COMPANY
.tables

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (1, 'Paul', 32, 'California', 20000.00 );
INSERT INTO COMPANY VALUES (7, 'James', 24, 'Houston', 10000.00 );

.header on
.mode column
SELECT * FROM COMPANY;


DROP TABLE COMPANY;
.tables

```

