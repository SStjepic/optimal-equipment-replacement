import os
import time
import subprocess
import pandas as pd
import numpy as np
import sys
from multiprocessing import cpu_count
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import generator

HORIZON = 30
SIMULATIONS_PER_MACHINE = 10000
REPEATS = 30

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PYTHON_REPORT_DIR = BASE_DIR
RUST_EXE = os.path.join(BASE_DIR, '..', '..', 'rust', 'parallel', 'target', 'release', 'parallel.exe')

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

def run_rust_simulation(input_file, cores, repeats=REPEATS):
    all_times = []

    for _ in range(repeats):
        start = time.perf_counter()
        subprocess.run([RUST_EXE, input_file, str(cores)], check=True)
        end = time.perf_counter()
        all_times.append(end - start)

    return all_times[1:]

def measure_scaling(cores_list, mode='Strong'):
    print(f"\n=== {mode.upper()} SCALING ===")
    results = []

    for cores in cores_list:
        if mode == 'Strong':
            n_machines = 100
            input_file = os.path.join(PYTHON_REPORT_DIR, 'strong_input.csv')
        else:
            n_machines = 10 * cores
            input_file = os.path.join(PYTHON_REPORT_DIR, f'weak_input_{cores}.csv')

        generator.generate_machines(n_machines, output_file=input_file)
        print(f"Cores: {cores} | Machines: {n_machines}")

        times = run_rust_simulation(input_file, cores)
        mean_t, std_t, outlier_count = calculate_metrics_iqr(times)

        results.append({
            'Language': 'Rust',
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
    weak_results = measure_scaling(cores_to_test, mode='Weak')

    df = pd.DataFrame(strong_results + weak_results)
    output_csv = os.path.join(PYTHON_REPORT_DIR, 'scaling_results_rust.csv')
    df.to_csv(output_csv, index=False)

    print(f"\nDone. Results saved to '{output_csv}'")
    print(df.to_string(index=False))