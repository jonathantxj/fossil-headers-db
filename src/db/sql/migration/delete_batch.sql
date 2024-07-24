DELETE FROM transactions
WHERE transaction_hash LIKE $1
;
    