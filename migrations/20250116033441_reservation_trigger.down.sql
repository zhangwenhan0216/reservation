DROP TRIGGER reservation_trigger ON rsvp.reservations;
DROP FUNCTION rsvp.reservation_trigger();
DROP TABLE rsvp.reservations_changes CASCADE;
DROP TYPE rsvp.reservation_update_type;