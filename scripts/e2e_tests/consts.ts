const UNIT = 10 ** 12; // ROC has 12 decimals

const INITIAL_PRICE = 50 * UNIT;
const CORE_COUNT = 10;
const TIMESLICE_PERIOD = 80;
const IDEAL_CORES_SOLD = 5;

const CONFIG = {
    advance_notice: 20,
    interlude_length: 0,
    leadin_length: 10,
    ideal_bulk_proportion: 0,
    limit_cores_offered: 50,
    region_length: 30,
    renewal_bump: 10,
    contribution_timeout: 5,
};

export {CONFIG, CORE_COUNT, IDEAL_CORES_SOLD, INITIAL_PRICE, TIMESLICE_PERIOD, UNIT};
