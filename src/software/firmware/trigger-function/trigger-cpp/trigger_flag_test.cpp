#include <iostream>
#include <fstream>
#include <algorithm> 
#include <vector>
#include <cmath>
#include <math.h>

#include "trigger_flag.h"

using namespace std;

int main()
{   
    vector<float> history; 
    vector<float> ma; 
    vector<float> diff;

    ifstream infile ("high.txt");
    
    if (infile.is_open())
    {
        ofstream outfile ("flag_high.txt");
        float pressure;
        while (infile >> pressure)
        {   
            outfile << trigger_flag(pressure, history, ma, diff) << "\n";
        }
        infile.close();
        outfile.close();
    }
    else
    {
        cout << "Error opening file";
    }

    return 0;
}

