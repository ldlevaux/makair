/******************************************************************************
 * @author Makers For Life
 * @copyright Copyright (c) 2020 Makers For Life
 * @file mass_flow_meter.cpp
 * @brief Mass Flow meter management
 *****************************************************************************/

/**
 * SFM3300-D sensirion mass flow meter is connected on I2C bus.
 * To perform the integral of the mass flow, i2c polling must be done in a high priority timer
 */

// Associated header
#include "../includes/mass_flow_meter.h"

// External
#include "../includes/config.h"
#include <Arduino.h>
#include <IWatchdog.h>
#include <OneButton.h>
#include <Wire.h>

// Internal
#include "../includes/buzzer_control.h"
#include "../includes/parameters.h"
#include "../includes/screen.h"

// Linked to Hardware v2
#ifdef MASS_FLOW_METER

// 2 khz => prescaler = 50000 => still OK for a 16 bit timer. it cannnot be slower
// 10 khz => nice
#define MASS_FLOW_TIMER_FREQ 10000

// the timer period in microsecond, 100us precision (because 10 khz prescale)
#define MASS_FLOW_PERIOD_US 1000
HardwareTimer* massFlowTimer;

volatile int32_t mfmAirVolumeSum = 0;
volatile int32_t mfmPreviousValue = 0;
volatile int32_t mfmSensorDetected = 0;

bool mfmFaultCondition = false;

// Time to reset the sensor after I2C restart, in periods. => 5ms.
#define MFM_WAIT_RESET_PERIODS 5
int32_t mfmResetStateMachine = MFM_WAIT_RESET_PERIODS;

union {
    unsigned short i;
    unsigned char c[2];
} mfmLastData;

void MFM_Timer_Callback(HardwareTimer*) {

    int32_t rawValue;
    int32_t newSum;

    if (!mfmFaultCondition) {
#if MODE == MODE_MFM_TESTS
        digitalWrite(PIN_LED_START, true);
#endif

#if MASS_FLOW_METER_SENSOR == MFM_SFM_3300D
        Wire.beginTransmission(MFM_SENSOR_I2C_ADDRESS);
        Wire.requestFrom(MFM_SENSOR_I2C_ADDRESS, 2);
        mfmLastData.c[1] = Wire.read();
        mfmLastData.c[0] = Wire.read();
        if (Wire.endTransmission() != 0) {  // If transmission failed
            mfmFaultCondition = true;
            mfmResetStateMachine = 5;
        }
        mfmAirVolumeSum += (int32_t)mfmLastData.i - 0x8000;
#endif

#if MODE == MODE_MFM_TESTS
        digitalWrite(PIN_LED_START, false);
#endif
    } else {

        if (mfmResetStateMachine == MFM_WAIT_RESET_PERIODS) {
            // reset attempt
            Wire.flush();
            Wire.end();
        }
        mfmResetStateMachine--;

        if (mfmResetStateMachine == 0) {
// MFM_WAIT_RESET_PERIODS cycles later, try again to init the sensor
#if MASS_FLOW_METER_SENSOR == MFM_SFM_3300D
            Wire.begin(true);
            Wire.beginTransmission(MFM_SENSOR_I2C_ADDRESS);
            Wire.write(0x10);
            Wire.write(0x00);
            mfmFaultCondition = (Wire.endTransmission() != 0);
#endif
            if (mfmFaultCondition) {
                mfmResetStateMachine = MFM_WAIT_RESET_PERIODS;
            }
        }
    }
}

/**
 *  Returns true if there is a Mass Flow Meter connected
 *  If not detected, you will always read volume = 0 mL
 */
boolean MFM_init(void) {

    mfmAirVolumeSum = 0;

    // set the timer
    massFlowTimer = new HardwareTimer(MASS_FLOW_TIMER);

    // prescaler. stm32f411 clock is 100mhz
    massFlowTimer->setPrescaleFactor((massFlowTimer->getTimerClkFreq() / MASS_FLOW_TIMER_FREQ) - 1);

    // set the period
    massFlowTimer->setOverflow(MASS_FLOW_TIMER_FREQ / MASS_FLOW_PERIOD_US);
    massFlowTimer->setMode(MASS_FLOW_CHANNEL, TIMER_OUTPUT_COMPARE, NC);
    massFlowTimer->attachInterrupt(MFM_Timer_Callback);

    // interrupt priority is documented here:
    // https://stm32f4-discovery.net/2014/05/stm32f4-stm32f429-nvic-or-nested-vector-interrupt-controller/
    massFlowTimer->setInterruptPriority(2, 0);

    // detect if the sensor is connected
    Wire.setSDA(PIN_I2C_SDA);
    Wire.setSCL(PIN_I2C_SCL);

    // init the sensor, test communication
#if MASS_FLOW_METER_SENSOR == MFM_SFM_3300D
    Wire.begin();  // join i2c bus (address optional for master)
    Wire.beginTransmission(MFM_SENSOR_I2C_ADDRESS);

    Wire.write(0x10);
    Wire.write(0x00);

    // mfmTimerCounter = 0;

    mfmFaultCondition = (Wire.endTransmission() != 0);

#endif

    massFlowTimer->resume();

    return !mfmFaultCondition;
}

/*
 * Reset the volume counter
 */
void MFM_reset(void) {
    mfmAirVolumeSum = 0;
    mfmPreviousValue = 0;
}

/**
 * return the number of milliliters since last reset
 * Can also perform the volume reset in the same atomic operation
 */
int32_t MFM_read_liters(boolean reset_after_read) {

    int32_t result;

#if MASS_FLOW_METER_SENSOR == MFM_SFM_3300D
    // this should be an atomic operation (32 bits aligned data)
    result = mfmFaultCondition ? 999999 : mfmAirVolumeSum / (60 * 120);

    // Correction factor is 120. Divide by 60 to convert ml.min-1 to ml.ms-1, hence the 7200 =
    // 120 * 60
#endif

    if (reset_after_read) {
        MFM_reset();
    }

    return result;
}

#if MODE == MODE_MFM_TESTS

void setup(void) {

    Serial.begin(115200);
    Serial.println("init mass flow meter");
    boolean ok = MFM_init();

    pinMode(PIN_SERIAL_TX, OUTPUT);
    pinMode(PIN_LED_START, OUTPUT);

    startScreen();
    resetScreen();
    screen.setCursor(0, 0);
    screen.print("debug prog");
    screen.setCursor(0, 1);
    screen.print("mass flow sensor");
    screen.setCursor(0, 2);
    screen.print(ok ? "sensor OK" : "sensor not OK");

    Serial.println("init done");
}

void loop(void) {

    delay(1000);

    char buffer[30];

    int32_t volume = MFM_read_liters(false);

    resetScreen();
    screen.setCursor(0, 0);
    screen.print("debug prog");
    screen.setCursor(0, 1);
    screen.print("mass flow sensor");
    screen.setCursor(0, 2);

    if (volume == MASS_FLOw_ERROR_VALUE) {
        screen.print("sensor not OK");
    } else {
        screen.print("sensor OK");
        screen.setCursor(0, 3);
        sprintf(buffer, "volume=%dmL", volume);
        screen.print(buffer);
    }

    Serial.print("volume = ");
    Serial.print(volume);
    Serial.println("mL");
}
#endif

#endif