# The overlay of ../accept_a_flexible_array_member_is_reached_by_tail.em.

unsafe overlay c "packets.h":
    fn packet_new(count) -> returns_owned(destructor=packet_free) one
    fn packet_sum(packet: borrowed one)
    hide packet_free
