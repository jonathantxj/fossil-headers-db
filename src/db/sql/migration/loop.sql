DO $$
DECLARE
    batch_size int := 10000;
    rows_affected int;
    total_rows_deleted int := 0;
    batch_count int := 0;
BEGIN
    LOOP
        BEGIN
            SELECT delete_batch(batch_size) INTO rows_affected;
            
            IF rows_affected = 0 THEN
                EXIT;
            ELSE
                COMMIT;
                
                total_rows_deleted := total_rows_deleted + rows_affected;
                batch_count := batch_count + 1;
                
                RAISE NOTICE 'Batch % completed: % rows deleted. Total rows deleted: %', 
                             batch_count, rows_affected, total_rows_deleted;
            END IF;
        END;
    END LOOP;
    
    RAISE NOTICE 'Deletion complete. Total batches: %. Total rows deleted: %', 
                 batch_count, total_rows_deleted;
END $$;