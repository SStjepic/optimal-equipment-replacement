import numpy as np
import pandas as pd
from multiprocessing import Pool, cpu_count
import time

def run_full_machine_sim(m_data, horizon_years, simulations, market_growth_rate):
    accumulated_profits = np.zeros(horizon_years)
    replacement_counts = {}

    for _ in range(simulations):
        current_age = m_data['initial_age']
        
        for year in range(horizon_years):
            market_factor = (1 + market_growth_rate) ** year
            revenue = (m_data['base_profit'] * ((1 - m_data['profit_decay']) ** current_age)) * market_factor
            maint = m_data['maint_base'] * (m_data['maint_growth'] ** current_age)
            
            if np.random.rand() < min(0.03 * current_age, 0.6):
                maint += m_data['repair_cost']

            yearly_profit = revenue - maint
            
            threshold = m_data['base_profit'] * 0.4
            if yearly_profit < threshold or maint > (revenue * 0.6):
                resale = m_data['resale_base'] * (m_data['resale_decay'] ** current_age)
                yearly_profit -= (m_data['buy_price'] - resale)
                current_age = 0
                replacement_counts[year + 1] = replacement_counts.get(year + 1, 0) + 1
            else:
                current_age += 1
            
            accumulated_profits[year] += yearly_profit

    typical_years = [str(y) for y, count in sorted(replacement_counts.items()) if count > (simulations * 0.2)]
    
    return {
        'id': m_data['id'],
        'mean_profits': accumulated_profits / simulations,
        'replacement_logs': ", ".join(typical_years)
    }

class MachineParallelSim:
    def __init__(self, input_file, horizon_years, simulations):
        self.df_machines = pd.read_csv(input_file)
        self.horizon_years = horizon_years
        self.simulations = simulations
        self.market_growth_rate = 0.01

    def run(self):
        num_cores = cpu_count()
        print(f"Starting parallel simulation by machines using {num_cores} cores")

        tasks = [row.to_dict() for _, row in self.df_machines.iterrows()]

        with Pool(processes=num_cores) as pool:
            results = pool.starmap(run_full_machine_sim, 
                                  [(m, self.horizon_years, self.simulations, self.market_growth_rate) for m in tasks])

        self.save_results(results)

    def save_results(self, results):
        results.sort(key=lambda x: x['id'])
        
        profit_rows = []
        all_profits_matrix = []
        
        for r in results:
            row = [int(r['id'])] + list(np.round(r['mean_profits'], 2))
            profit_rows.append(row)
            all_profits_matrix.append(r['mean_profits'])
        
        system_total = np.round(np.sum(all_profits_matrix, axis=0), 2)
        profit_rows.append(['SYSTEM_TOTAL'] + list(system_total))
        
        cols = ['machine_id'] + [f'year_{i+1}_profit' for i in range(self.horizon_years)]
        pd.DataFrame(profit_rows, columns=cols).to_csv("machine_profit_parallel.csv", index=False)

        repl_rows = [{'machine_id': int(r['id']), 'replacement_years': r['replacement_logs']} for r in results]
        pd.DataFrame(repl_rows).to_csv("machine_replacements_parallel.csv", index=False)
        print("Results saved to 'machine_profit_parallel.csv' and 'machine_replacements_parallel.csv'.")

if __name__ == "__main__":
    sim = MachineParallelSim('machines_input.csv', horizon_years=30, simulations=10000)
    sim.run()