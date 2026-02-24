import sys
import os
import time
import pandas as pd
import numpy as np
from multiprocessing import cpu_count

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import generator
from parallel import EquipmentReplacementParallelSim

HORIZON = 30
SIMULATIONS_PER_MACHINE = 10000
REPEATS = 31

def calculate_metrics_iqr(times):
    times_arr = np.array(times)
    q1 = np.percentile(times_arr, 25)
    q3 = np.percentile(times_arr, 75)
    iqr = q3 - q1
    
    lower_bound = q1 - 1.5 * iqr
    upper_bound = q3 + 1.5 * iqr
    
    outliers_mask = (times_arr < lower_bound) | (times_arr > upper_bound)
    clean_times = times_arr[~outliers_mask]
    
    return np.mean(clean_times), np.std(clean_times), np.sum(outliers_mask)

def run_simulation(input_file, cores, repeats=REPEATS):
    all_times = []
    for i in range(repeats):
        start = time.perf_counter()
        sim = EquipmentReplacementParallelSim(input_file, HORIZON, SIMULATIONS_PER_MACHINE)
        sim.run(num_cores=cores, save=True)
        end = time.perf_counter()
        
        duration = end - start
        all_times.append(duration)
    
    return all_times[1:]

def measure_scaling(cores_list, mode='Strong'):
    print(f"\n=== {mode.upper()} SCALING ===")
    results = []
    
    for cores in cores_list:
        print(results)
        if mode == 'Strong':
            n_machines = 100
            input_file = os.path.join('', 'strong_input.csv')
        else:
            n_machines = 10 * cores
            input_file = os.path.join('', f'weak_input_{cores}.csv')
            
        generator.generate_machines(n_machines, output_file=input_file)
        print(f"Cores: {cores} | Machines: {n_machines}\n")
        
        times = run_simulation(input_file, cores)
        mean_t, std_t, outlier_count = calculate_metrics_iqr(times)
        
        results.append({
            'Language': 'Python',
            'Mode': mode,
            'Cores': cores,
            'Simulations': n_machines * SIMULATIONS_PER_MACHINE,
            'Mean_Time_s': mean_t,
            'Std_Dev': std_t,
            'Outliers_IQR': outlier_count
        })
    return results

if __name__ == "__main__":
    max_cores = cpu_count()
    cores_to_test = [c for c in range(1, 9) if c <= max_cores]
    
    strong_results = measure_scaling(cores_to_test, mode='Strong')
    print(strong_results)
    weak_results = measure_scaling(cores_to_test, mode='Weak')
    
    df = pd.DataFrame(strong_results + weak_results)
    
    output_csv = 'scaling_results_python.csv'
    df.to_csv(output_csv, index=False)
    
    print(f"\nDone. Results saved to '{output_csv}'")
    print(df.to_string(index=False))