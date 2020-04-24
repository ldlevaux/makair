#ifndef TRIGGER_FLAG_H_INCLUDED
#define TRIGGER_FLAG_H_INCLUDED

#include <vector>

/*
* flag=0 : rien à signaler
* flag=1 : début expiration
* flag=2 : début inspiration automatique
* flag=3 : début inspiration voulue par le patient
*/
int trigger_flag(
    float pressure, //(mmH2O)
    std::vector<float>& history, 
    std::vector<float>& ma, 
    std::vector<float>& diff, 
    float lim_inf_plateau = 180, //(mmH2O)
    float lim_sup_before_inspi = 150, //(mmH2O)
    float lim_diff = 3 //(mmH2O)
);

#endif // TRIGGER_FLAG_H_INCLUDED