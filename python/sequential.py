import numpy as np
import pandas as pd

class EquipmentReplacementSim:
    def __init__(self, input_file, horizon_years, simulations):
        self.df_machines = pd.read_csv(input_file)
        self.num_machines = len(self.df_machines)
        self.horizon_years = horizon_years 
        self.simulations = simulations
        self.machines_data = self.df_machines.to_dict('records')
        self.market_growth_rate = 0.01

    def get_revenue(self, age, machine, current_sim_year):
        market_factor = (1 + self.market_growth_rate) ** current_sim_year
        return (machine['base_profit'] * ((1 - machine['profit_decay']) ** age)) * market_factor

    def get_maint_cost(self, age, machine):
        return machine['maint_base'] * (machine['maint_growth'] ** age)

    def get_resale_value(self, age, machine):
        return machine['resale_base'] * (machine['resale_decay'] ** age)

    def get_breakdown_prob(self, age):
        return min(0.03 * age, 0.6)
    
    def run_monte_carlo(self):
        machine_profits = np.zeros((self.num_machines, self.horizon_years))
        all_replacement_events = []

        for m_idx in range(self.num_machines):
            m = self.machines_data[m_idx]
            accumulated_profits_per_year = np.zeros(self.horizon_years)
            replacement_years_count = {}

            for _ in range(self.simulations):
                current_age = m['initial_age']
                current_m = m.copy()
                
                for year in range(self.horizon_years):
                    revenue = self.get_revenue(current_age, current_m, year)
                    maint = self.get_maint_cost(current_age, current_m)
                    
                    if np.random.rand() < self.get_breakdown_prob(current_age):
                        maint += current_m['repair_cost']

                    yearly_profit = revenue - maint
                    
                    threshold = current_m['base_profit'] * 0.4
                    if yearly_profit < threshold or maint > (revenue * 0.6):
                        net_cost = current_m['buy_price'] - self.get_resale_value(current_age, current_m)
                        yearly_profit -= net_cost
                        
                        current_age = 0
                        replacement_years_count[year + 1] = replacement_years_count.get(year + 1, 0) + 1
                    else:
                        current_age += 1
                    
                    accumulated_profits_per_year[year] += yearly_profit

            typical_years = [str(y) for y, count in sorted(replacement_years_count.items()) if count > (self.simulations * 0.2)]
            all_replacement_events.append(", ".join(typical_years))
            
            machine_profits[m_idx] = accumulated_profits_per_year / self.simulations

        return all_replacement_events, machine_profits

    def save_results(self, replacement_logs, machine_profits):
        cols_profit = ['machine_id'] + [f'year_{i+1}_profit' for i in range(self.horizon_years)]
        profit_data = []
        for i in range(self.num_machines):
            row = [int(self.machines_data[i]['id'])] + list(np.round(machine_profits[i], 2))
            profit_data.append(row)
        
        system_total = np.round(np.sum(machine_profits, axis=0), 2)
        profit_data.append(['SYSTEM_TOTAL'] + list(system_total))
        
        pd.DataFrame(profit_data, columns=cols_profit).to_csv("machine_profit_sequential.csv", index=False)

        replacement_data = []
        for i in range(self.num_machines):
            replacement_data.append({
                'machine_id': int(self.machines_data[i]['id']),
                'replacement_years': replacement_logs[i]
            })
        pd.DataFrame(replacement_data).to_csv("machine_replacements_sequential.csv", index=False)
        print("Results saved to 'machine_profit_sequential.csv' and 'machine_replacements_sequential.csv'.")

if __name__ == "__main__":
    sim = EquipmentReplacementSim('machines_input.csv', horizon_years=30, simulations=10000)
    logs, profits = sim.run_monte_carlo()
    sim.save_results(logs, profits)