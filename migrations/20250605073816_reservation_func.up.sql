-- Add up migration script here
CREATE OR REPLACE FUNCTION rsvp.query(
    uid text,
    rid text,
    status rsvp.reservation_status,
    during TSTZRANGE,
    page integer DEFAULT 1,
    page_size integer DEFAULT 10,
    is_desc bool DEFAULT FALSE
) RETURNS TABLE (LIKE rsvp.reservations) AS $$
DECLARE
    _sql text;
BEGIN
    IF page_size < 10 OR page_size > 100 THEN
        page_size := 10; -- Default page size
    END IF;
    IF page < 1 THEN
        page := 1;
    END IF;

    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND status = %L AND %s ORDER BY lower(timespan) %s LIMIT %L::integer OFFSET %L::integer', 
        during,
        status,
        CASE
            WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
            WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
            WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
            ELSE 'resource_id = ' || quote_literal(rid) || ' AND user_id = ' || quote_literal(uid)
        END,
        CASE WHEN is_desc THEN 'DESC' ELSE 'ASC' END,
        page_size, (page - 1) * page_size
    );

    -- log _sql;
    RAISE NOTICE 'Executing SQL: %', _sql;

    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;