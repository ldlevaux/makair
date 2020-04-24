# %%
import pandas as pd
import numpy as np
import plotly
import plotly.graph_objs as go

high_spontaneous_breathing = pd.read_csv("../pressure_with_high_spontaneous_breathing.txt", header=2, usecols=[0,2], names=['time', 'pressure'])
low_spontaneous_breathing = pd.read_csv("../pressure_with_low_spontaneous_breathing.txt", header=2, usecols=[0,2], names=['time', 'pressure'])
no_spontaneous_breathing = pd.read_csv("../pressure_with_no_spontaneous_breathing.txt", header=2, usecols=[0,2], names=['time', 'pressure'])

series = high_spontaneous_breathing.copy()
#series = series.loc[:3000]

# Clean the series
series.dropna(axis=0, inplace=True)
series.reset_index(inplace=True, drop=True)

#np.savetxt('high.txt', series['pressure'].values)


def trigger_flag(pressure, history, ma, diff, lim_inf_plateau = 180, lim_sup_before_inspi = 150, lim_diff = 3):
    history = np.append(history, pressure)
    
    if len(history) >= 2:
        # Compute a moving average
        ma = np.append(ma, (history[-1]+history[-2])/2)
    
    if len(ma) >= 2:
        # Compute difference between moving average
        diff = np.append(diff, (ma[-1]-ma[-2]))
    
    flag = "normal"
    if len(diff) > 20:
        # we flag each diff > 3 mmH2O if the 20 previous diffs are <=3
        if abs(diff[-1]) > lim_diff and all(abs(diff[-21:-1]) <= lim_diff):
            # expi
            if history[-2] > lim_inf_plateau:
                flag = "expi"
            # inspi auto    
            elif history[-2] < lim_sup_before_inspi and diff[-1] > 0: 
                flag = "inspi_auto"
            # inspi patient
            elif history[-2] < lim_sup_before_inspi and diff[-1] < 0:
                flag = "inspi_patient"

    return len(diff), flag, history, ma, diff

# Parameters
history = np.array([])
ma = np.array([])
diff = np.array([0, 0])

flag_expi = np.array([])
flag_inspi_auto = np.array([])
flag_inspi_patient = np.array([])

for pressure in series['pressure'].values:
    i, flag, history, ma, diff = trigger_flag(pressure, history, ma, diff)
    
    if flag=="expi":
        flag_expi = np.append(flag_expi, i)
    elif flag=="inspi_auto":
        flag_inspi_auto = np.append(flag_inspi_auto, i)

    elif flag=="inspi_patient":
        flag_inspi_patient = np.append(flag_inspi_patient, i)



#print(flag_expi)
#print(flag_inspi_auto)
#print(flag_inspi_patient)




fig = go.Figure()
fig.add_trace(
    go.Scatter(name="series", x=series['time'], y=series['pressure'])
)

fig.add_trace(
    go.Scatter(name="ma", x=series['time'], y=np.concatenate(([np.nan], ma)))
)
fig.update_layout(height=400, width=1400)
fig.show()

fig = go.Figure()
fig.add_trace(
    go.Scatter(name="diff", x=series['time'], y=diff)
)
fig.add_trace(
    go.Scatter(name="diff > lim_diff", x=series['time'], y=diff*(abs(diff)>lim_diff))
)
fig.update_layout(height=400, width=1400)
fig.show()


fig = go.Figure()
fig.add_trace(
    go.Scatter(name="series", x=series['time'], y=series['pressure'])
)

fig.add_trace(
    go.Scatter(
        name='flag_expi', 
        x=series.loc[flag_expi, 'time'],
        y=series['pressure'].loc[flag_expi],
        mode='markers', 
        marker=dict(
        size=8,
        color='red',
        symbol='cross'
        )
   )
)

fig.add_trace(
    go.Scatter(
        name='flag_inspi_auto', 
        x=series.loc[flag_inspi_auto, 'time'],
        y=series['pressure'].loc[flag_inspi_auto],
        mode='markers', 
        marker=dict(
        size=8,
        color='green',
        symbol='cross'
        )
   )
)

fig.add_trace(
    go.Scatter(
        name='flag_inspi_patient', 
        x=series.loc[flag_inspi_patient, 'time'],
        y=series['pressure'].loc[flag_inspi_patient],
        mode='markers', 
        marker=dict(
        size=8,
        color='purple',
        symbol='cross'
        )
   )
)


fig.update_layout(height=400, width=1400)
fig.show()



# %%
