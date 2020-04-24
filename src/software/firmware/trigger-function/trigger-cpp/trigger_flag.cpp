#include "trigger_flag.h"

#include <iostream>
#include <fstream>
#include <algorithm> 
#include <vector>
#include <cmath>
#include <math.h>

using namespace std;


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
)
{
    history.push_back(pressure);

    if (history.size() >= 2)
    {
        ma.push_back((history[history.size()-1] + history[history.size()-2]) / 2);
    }

    if (ma.size() >= 2)
    {
        diff.push_back((ma[ma.size()-1] - ma[ma.size()-2]));
    } else {
        diff.push_back(0);
    }
    
    int flag = 0;
    if (diff.size() > 20)
    {   
        if (fabs(diff[diff.size()-1]) > lim_diff && std::all_of(diff.end()-20, diff.end()-1, [lim_diff](float d){return fabs(d) <= lim_diff;}) )
        {  
            if (history[history.size()-2] > lim_inf_plateau)
            {
                flag = 1;
            } else if (history[history.size()-2] < lim_sup_before_inspi && diff[diff.size()-1] > 0)
            {
                flag = 2;
            }
            else if (history[history.size()-2] < lim_sup_before_inspi && diff[diff.size()-1] < 0)
            {
                flag = 3;
            }
            
        }
    }

    return flag;
}