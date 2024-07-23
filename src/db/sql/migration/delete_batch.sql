CREATE OR REPLACE FUNCTION delete_batch(batch_size int)
RETURNS int AS $$
DECLARE
    rows_affected int;
BEGIN
    DELETE FROM transactions
    WHERE transaction_hash IN (
        SELECT transaction_hash
        FROM transactions
        LIMIT batch_size
    );
    
    GET DIAGNOSTICS rows_affected = ROW_COUNT;
    RETURN rows_affected;
END;
$$ LANGUAGE plpgsql;