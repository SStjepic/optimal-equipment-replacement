import pandas as pd
import numpy as np

def generate_machines(num_machines=1000):
    data = []
    for i in range(num_machines):
        initial_age = np.random.randint(0, 7)
        base_profit = np.random.uniform(4000, 7000) 
        
        buy_price = base_profit * 1.5

        profit_decay = np.random.uniform(0.08, 0.12)

        maint_base = np.random.uniform(300, 600)

        maint_growth = np.random.uniform(1.05, 1.10)

        resale_base = buy_price * 0.6

        resale_decay = np.random.uniform(0.75, 0.85)

       

        repair_cost = np.random.uniform(800, 2000)
        
        data.append([
            i, initial_age, round(base_profit, 2), round(profit_decay, 3),
            round(maint_base, 2), round(maint_growth, 3),
            round(resale_base, 2), round(resale_decay, 3),
            round(buy_price, 2), round(repair_cost, 2)
        ])
    
    columns = [
        'id', 'initial_age', 'base_profit', 'profit_decay', 
        'maint_base', 'maint_growth', 'resale_base', 
        'resale_decay', 'buy_price', 'repair_cost'
    ]
    
    df = pd.DataFrame(data, columns=columns)
    df.to_csv('machines_input.csv', index=False)
    print("File 'machines_input.csv' has been successfully generated.")

if __name__ == "__main__":
    generate_machines(100)