CREATE TYPE rsvp.reservation_update_type AS ENUM ('unknown', 'create', 'update', 'delete');

CREATE TABLE rsvp.reservations_changes (
    id SERIAL NOT NULL,
    reservation_id BIGSERIAL NOT NULL,
    old JSONB,
    new JSONB,
    op rsvp.reservation_update_type NOT NULL,
    
    CONSTRAINT reservation_changes_pkey PRIMARY KEY (id)
);

CREATE INDEX reservations_changes_reservation_id_op_idx ON rsvp.reservations_changes (reservation_id, op);

CREATE OR REPLACE FUNCTION rsvp.reservation_trigger() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO rsvp.reservations_changes (reservation_id, old, new, op) VALUES (NEW.id, null, to_jsonb(NEW), 'create');
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.status <> NEW.status THEN
            INSERT INTO rsvp.reservations_changes (reservation_id, old, new, op) VALUES (NEW.id, to_jsonb(OLD), to_jsonb(NEW), 'update');
        END IF;
   ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO rsvp.reservations_changes (reservation_id, old, new, op)
        VALUES (OLD.id, to_jsonb(OLD), null, 'delete');
    END IF;
    NOTIFY reservation_update;
    RETURN NULL; 
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservation_trigger AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations FOR EACH ROW EXECUTE PROCEDURE rsvp.reservation_trigger();