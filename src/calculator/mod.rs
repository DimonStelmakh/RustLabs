use std::io;

pub fn run_calculator() {
    let mut memory: Option<f64> = None;

    println!("\nЛаскаво просимо до калькулятора!\nОкрім виразів, можна написати 'exit' для виходу \
        або скористатися словом 'mem' замість числа, щоб взяти результат попереднього обчислення\n");
    loop {
        println!("Введіть вираз:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Не вдалося зчитати введення");

        let input = input.trim();

        if input.to_lowercase() == "exit" {
            break;
        }

        // пробуємо обчислити
        match evaluate_expression(&input, &memory) {
            Ok(result) => {
                println!("Результат: {}", result);
                memory = Some(result);  // зберігаємо в пам'ять
            }
            Err(e) => {
                println!("Помилка: {}", e);
            }
        }
    }
}

fn evaluate_expression(input: &str, memory: &Option<f64>) -> Result<f64, &'static str> {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    if tokens.len() != 3 {
        return Err("Невірний формат. Використовуйте: число оператор число (наприклад, 3 + 2 або mem + 2). \
        Пробіли мають значення");
    }

    let num1: f64 = if tokens[0] == "mem" {
        match memory {
            Some(value) => *value,
            None => {
                return Err("Пам'ять порожня");
            }
        }
    } else {
        tokens[0].parse::<f64>().expect("Невірний формат числа")
    };

    let operator = tokens[1];
    let num2: f64 = tokens[2].parse::<f64>().expect("Невірний формат числа");

    match operator {
        "+" => Ok(num1 + num2),
        "-" => Ok(num1 - num2),
        "*" => Ok(num1 * num2),
        "/" => {
            if num2 == 0.0 {
                Err("Ділення на нуль не допускається")
            } else {
                Ok(num1 / num2)
            }
        }
        _ => Err("Невідомий оператор"),
    }
}
