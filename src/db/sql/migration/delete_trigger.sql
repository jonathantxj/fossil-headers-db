CREATE TRIGGER delete_trigger
BEFORE DELETE ON transactions
FOR EACH ROW
EXECUTE FUNCTION copy_deleted_record();