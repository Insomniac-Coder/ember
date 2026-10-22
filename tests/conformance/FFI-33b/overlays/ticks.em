# The overlay of ../run_fail_a_creator_callback_called_on_another_thread.em and its accept case.

unsafe overlay c "ticks.h":
    fn tick_here(tick: callback(borrowed, user_data=data) threads(creator), data)
    fn tick_elsewhere(tick: callback(borrowed, user_data=data) threads(creator), data)
