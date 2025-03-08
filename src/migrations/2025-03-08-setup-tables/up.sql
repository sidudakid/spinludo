```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    balance NUMERIC(10, 2) NOT NULL DEFAULT 0
);

CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    player1_id INT NOT NULL REFERENCES users(id),
    player2_id INT REFERENCES users(id),
    entry_fee NUMERIC(10, 2) NOT NULL,
    owner_cut NUMERIC(5, 2) NOT NULL,
    status VARCHAR(20) NOT NULL
);
```
