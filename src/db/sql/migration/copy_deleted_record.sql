CREATE OR REPLACE FUNCTION copy_deleted_record()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO transactions2(block_number, transaction_hash, transaction_index, from_addr, to_addr, value, gas_price, max_priority_fee_per_gas, max_fee_per_gas, gas, chain_id)
    VALUES (OLD.block_number, OLD.transaction_hash, OLD.transaction_index, OLD.from_addr, OLD.to_addr, OLD.value, OLD.gas_price, OLD.max_priority_fee_per_gas, OLD.max_fee_per_gas, OLD.gas, OLD.chain_id);
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;