/*=============================================================================
 * @file pression.cpp
 *
 * COVID Respirator
 *
 * @section copyright Copyright
 *
 * Makers For Life
 *
 * @section descr File description
 *
 * Fichier définissant les fonctionnalités liées à l'acquisition ou la simulation
 * du capteur de pression
 */

#pragma once

// INCLUDES ==================================================================

// External
#include <Arduino.h>

// Internal
#include "parameters.h"

// PROGRAM =====================================================================

double filteredVout = 0;
const double RATIO_PONT_DIVISEUR = 0.8192;
const double V_SUPPLY = 5.08;
const double KPA_MMH2O = 101.97162129779;

// Get the measured or simulated pressure for the feedback control
#ifdef SIMULATION
int readPressureSensor(uint16_t centiSec)
{
    if (centiSec < uint16_t(50))
    {
        pController.updatePressure(600);
    }
    else if (centiSec < uint16_t(100))
    {
        pController.updatePressure(300);
    }
    else if (centiSec < 200)
    {
        pController.updatePressure(110);
    }
    else if  (centiSec < 250)
    {
        pController.updatePressure(90);
    }
    else
    {
        pController.updatePressure(70);
    }
}
#else
int readPressureSensor(uint16_t centiSec)
{
    double rawVout = analogRead(PIN_CAPTEUR_PRESSION) * 3.3 / 1024.0;
    filteredVout = filteredVout + (rawVout - filteredVout) * 0.2;

    // Ratio a cause du pont diviseur
    double vOut = filteredVout / RATIO_PONT_DIVISEUR;

    // Pression en kPA
    double pressure  = (vOut / V_SUPPLY - 0.04) / 0.09;

    if (pressure <= 0.0)
    {
        return 0;
    }
    return pressure * KPA_MMH2O;
}
#endif