import numpy as np
import pandas as pd
from multiprocessing import Pool, cpu_count
from numba import njit
import time

@njit
def run_machine_numba(
    initial_age, base_profit, profit_decay,
    maint_base, maint_growth,
    resale_base, resale_decay,
    buy_price, repair_cost,
    horizon_years, simulations,
    market_growth_rate
):

    accumulated_profits = np.zeros(horizon_years)
    replacement_counts = np.zeros(horizon_years)

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

def run_machine_sim(args):
    m_data, horizon_years, simulations, market_growth_rate = args

    profits, replacement_counts = run_machine_numba(
        m_data['initial_age'],
        m_data['base_profit'],
        m_data['profit_decay'],
        m_data['maint_base'],
        m_data['maint_growth'],
        m_data['resale_base'],
        m_data['resale_decay'],
        m_data['buy_price'],
        m_data['repair_cost'],
        horizon_years,
        simulations,
        market_growth_rate
    )

    typical_years = [
        str(i + 1)
        for i in range(horizon_years)
        if replacement_counts[i] > simulations * 0.2
    ]

    return {
        'id': m_data['id'],
        'mean_profits': profits,
        'replacement_logs': ", ".join(typical_years)
    }


class EquipmentReplacementParallelSim:

    def __init__(self, input_file, horizon_years, simulations):
        self.df_machines = pd.read_csv(input_file)
        self.horizon_years = horizon_years
        self.simulations = simulations
        self.market_growth_rate = 0.01

    def run(self):

        num_cores = cpu_count()
        print(f"Starting parallel simulation using {num_cores} cores")

        tasks = [
            (row.to_dict(), self.horizon_years, self.simulations, self.market_growth_rate)
            for _, row in self.df_machines.iterrows()
        ]
        
        with Pool(processes=num_cores) as pool:
            results = pool.map(run_machine_sim, tasks)

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

        cols = ['machine_id'] + [
            f'year_{i+1}_profit' for i in range(self.horizon_years)
        ]

        pd.DataFrame(profit_rows, columns=cols)\
            .to_csv("machine_profit_parallel.csv", index=False)

        repl_rows = [
            {'machine_id': int(r['id']),
             'replacement_years': r['replacement_logs']}
            for r in results
        ]

        pd.DataFrame(repl_rows)\
            .to_csv("machine_replacements_parallel.csv", index=False)

        print("Results saved to 'machine_profit_parallel.csv' and 'machine_replacements_parallel.csv'.")


if __name__ == "__main__":
    simulation = EquipmentReplacementParallelSim('machines_input.csv',horizon_years=30, simulations=10000)
    simulation.run()
