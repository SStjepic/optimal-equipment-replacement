import numpy as np
import pandas as pd
from multiprocessing import Pool, cpu_count

def simulate_lifecycle(
    initial_age, base_profit, profit_decay,
    maint_base, maint_growth,
    resale_base, resale_decay,
    buy_price, repair_cost,
    horizon_years, simulations,
    market_growth_rate
):
    accumulated_profits = np.zeros(horizon_years, dtype=np.float64)
    replacement_counts = np.zeros(horizon_years, dtype=np.float64)

    for _ in range(simulations):
        current_age = initial_age
        for year in range(horizon_years):
            market_factor = (1 + market_growth_rate) ** year
            revenue = base_profit * ((1 - profit_decay) ** current_age) * market_factor
            maint = maint_base * (maint_growth ** current_age)

            if np.random.random() < min(0.03 * current_age, 0.6):
                maint += repair_cost

            yearly_profit = revenue - maint
            threshold = base_profit * 0.4

            if yearly_profit < threshold or maint > revenue * 0.6:
                resale = resale_base * (resale_decay ** current_age)
                yearly_profit -= (buy_price - resale)
                current_age = 0
                replacement_counts[year] += 1
            else:
                current_age += 1

            accumulated_profits[year] += yearly_profit

    return accumulated_profits / simulations, replacement_counts

def execute_simulation(args):
    m_values, horizon_years, simulations, market_growth_rate = args
    
    profits, replacement_counts = simulate_lifecycle(
        m_values[1], m_values[2], m_values[3], m_values[4], 
        m_values[5], m_values[6], m_values[7], m_values[8], 
        m_values[9], horizon_years, simulations, market_growth_rate
    )

    typical_years = [str(i + 1) for i in range(horizon_years) if replacement_counts[i] > simulations * 0.2]

    return (int(m_values[0]), profits, ", ".join(typical_years))

class EquipmentReplacementParallelSim:
    def __init__(self, input_file, horizon_years, simulations):
        self.df_machines = pd.read_csv(input_file)
        self.horizon_years = horizon_years
        self.simulations = simulations
        self.market_growth_rate = 0.01

    def run(self, num_cores=None, save = True):
        if num_cores is None:
            num_cores = cpu_count()
        
        data_matrix = self.df_machines.values 
        
        tasks = [
            (data_matrix[i], self.horizon_years, self.simulations, self.market_growth_rate)
            for i in range(len(data_matrix))
        ]
        
        with Pool(processes=num_cores) as pool:
            results = pool.map(execute_simulation, tasks)
            
        if save:
            self.save_results(results)

    def save_results(self, results):
        results.sort(key=lambda x: x[0])
        
        ids = [r[0] for r in results]
        profits = np.array([r[1] for r in results])
        logs = [r[2] for r in results]

        system_total = np.round(np.sum(profits, axis=0), 2)
        cols = ['machine_id'] + [f'year_{i+1}_profit' for i in range(self.horizon_years)]
        
        df_profit = pd.DataFrame(np.round(profits, 2), columns=cols[1:])
        df_profit.insert(0, 'machine_id', ids)
        
        total_row = pd.DataFrame([['SYSTEM_TOTAL'] + list(system_total)], columns=cols)
        pd.concat([df_profit, total_row]).to_csv("machine_profit_parallel.csv", index=False)

        pd.DataFrame({'machine_id': ids, 'replacement_years': logs})\
            .to_csv("machine_replacements_parallel.csv", index=False)

if __name__ == "__main__":
    sim = EquipmentReplacementParallelSim('machines_input.csv', 30, 10000)
    sim.run()