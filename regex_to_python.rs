// This program takes an input file with regex parsing and mapping instructions, and outputs corresponding Python code.

use std::fs;

fn main() {

    let mut log_vector = vec![];

    let filename = ".\\sample_input.txt";

    // Print text to the console
    println!("Processing Input File: {}", filename);
    println!("### Start of Output");

    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    let mut idx = 0;
    for line in contents.lines() {

        //split line columns
        let line_cols: Vec<&str> = line.split('\t').collect();

        let line_col = line_cols[0].trim();
        let regex_col = line_cols[1].trim();
        let text_col = line_cols[2].trim();
        let action_col = line_cols[3].trim();
        let mercy_col = line_cols[4].trim();
        let operation_type_col = line_cols[6].trim();

        println!("        if not matchedYet:");
        let mut count_left = 0;
        let mut count_right = 0;
        let mut line_pos = 0;
        let mut variable_left_delims = vec![];
        let mut variable_right_delims = vec![];


        //PARSE VARIABLES IN REGEX (Input Column #2)
        for c in regex_col.chars(){
            if c == '<'{
                count_left += 1;
                variable_left_delims.push(line_pos);
            }
            else if c == '>'{
                count_right += 1;
                variable_right_delims.push(line_pos);
            }
            line_pos += 1;
        }

        let mut variables = vec![];
        let mut new_line = String::from(regex_col);
        if count_left == count_right && count_left != 0{

            //build variables vector and new regex line
            for i in 0..count_left{
                variables.push(&regex_col[variable_left_delims[i]..variable_right_delims[i]+1]);
                let string_to_replace = &regex_col[variable_left_delims[i]..variable_right_delims[i]+1];
                let replaced_with = &("P".to_owned()+&regex_col[variable_left_delims[i]..variable_right_delims[i]+1]);
                new_line = new_line.replace(string_to_replace,replaced_with);
            }

            print!("            match = re.search(r'");
            print!("{}", new_line);
            println!("', data)");
            println!("            if match :");
            println!("                # {} rd fields", count_left);
             
            for var in &variables{
                let var_name = &var[1..var.chars().count()-1]; //remove angle brackets
                println!("                if match.group('{}') is not None: rd_fields['{}'] = match.group('{}').strip()", var_name, var_name, var_name);
            }
        }
        else {
            print!("            match = re.search(r'");
            print!("{}", regex_col);
            println!("', data)");
            println!("            if match :");
        }

        println!("                attrs['LINE_NAME'] = '{}'", line_col); 


        //PARSE VARIABLES IN TEXT (Input Column #3)
        let mut variables2 = vec![];
        let mut constants2 = vec![];
        let mut variables2_checked = vec![];

        let mut no_funcs = true;
        for c in text_col.chars(){
            if c == '('{
                no_funcs = false;
            }
        }

        if text_col.chars().count() != 0 && no_funcs{
            let tmp = text_col.replace("+  ","+").replace(" + ","+").replace(" +","+").replace("+ ","+").replace("+","><"); //assumes a plus sign is never a string literal
            let item_text = "<".to_owned()+&tmp+&">".to_owned();
            count_left = 0;
            count_right = 0;
            line_pos = 0;
            variable_left_delims = vec![];
            variable_right_delims = vec![];
            for c in item_text.chars(){       
                if c == '<'{
                    count_left += 1;
                    variable_left_delims.push(line_pos);
                }
                else if c == '>'{
                    count_right += 1;
                    variable_right_delims.push(line_pos);
                }
                line_pos += 1;
            }

            if count_left == count_right && count_left != 0{
                for i in 0..count_left{
                    let vv = &item_text[variable_left_delims[i]..variable_right_delims[i]+1];
                    let mut vv_str = vv.to_string();
                    if vv.chars().nth(1) != Some('\'') && vv.chars().nth(vv.chars().count()-2) != Some('\'') {
                        variables2.push((vv.to_lowercase(),false,vv)); //keep lowercase variable, and original case variable
                    }
                    else {
                        if vv.chars().nth(1) == Some('\'') && vv.chars().nth(vv.chars().count()-2) == Some('\'') {
                            //constant is properly delimited with quotes on both ends; no need to pad.
                        }
                        else if vv.chars().nth(1) == Some('\''){ //pad with right quote
                            vv_str = vv_str+"\'";
                            log_vector.push((line_col,"has badly delimited constant: ",vv.to_string()));
                        }
                        else { //pad with left quote
                            vv_str = "\'".to_owned()+&vv_str;
                            log_vector.push((line_col,"has badly delimited constant: ",vv.to_string()));
                        }
                        constants2.push(vv_str);
                    }
                }

                //check that variables2 are in variables of the regex
                for mut v in variables2{
                    for w in &variables{
                        if v.0 == w.to_string(){
                            v.1 = true;
                        }
                    }
                    if v.1 == false{
                        log_vector.push((line_col,"has invalid variables",v.0.to_string()));
                    }
                    variables2_checked.push(v);
                }
            }
            else {
                println!("################PROBLEM: Variables expected but could not be parsed.");
            }

            let mut new_item_text = item_text.to_string();
            for v in &variables2_checked{
                //each variable needs to look like this: rd_fields['x']
                let new_var = v.0.replace("<","<rd_fields[\'").replace(">","\']>").to_string();
                new_item_text = new_item_text.replace(v.2,&new_var);
            }

            new_item_text = new_item_text.replace("><"," + ").replace("<","").replace(">","");

            if new_item_text[0..9] != "rd_fields".to_string() && new_item_text.chars().nth(0) != Some('\''){
                new_item_text = "'".to_owned()+&new_item_text;
            }

            println!("                attrs['ITEM_TEXT'] = {}", new_item_text);
        }
        else if !no_funcs{
            println!("                attrs['ITEM_TEXT'] = '' #TBD# NEEDS MANUAL WORK #TBD#");
        }
        else {
            println!("                attrs['ITEM_TEXT'] = ''");
        }

        
        //HANDLE ACTION (Input Column #4)
        if action_col == "Ironclad" || action_col == "Halcyon" || action_col == "Ignore"{
            println!("                attrs['ACTION'] = '{}'", action_col);
        }
        else {
            let new_action = action_col.replace(")","").replace("(","<").replace(" = ","=").replace("= ","=").replace(" =","=").replace("=",">");
            variable_left_delims = vec![];
            variable_right_delims = vec![];
            line_pos = 0;
            for c in new_action.chars(){       
                if c == '<'{
                    variable_left_delims.push(line_pos);
                }
                else if c == '>'{
                    variable_right_delims.push(line_pos);
                }
                line_pos += 1;
            }

            let if_var = &new_action[variable_left_delims[0]..variable_right_delims[0]+1];
            let mut variables3 = vec![];
            let mut variables3_checked = vec![];
            variables3.push((if_var,false));

            //check that variables3 are in variables of the regex
            for mut v in variables3{
                for w in &variables{
                    if v.0 == w.to_string(){
                        v.1 = true;
                    }
                }
                if v.1 == false{
                    log_vector.push((line_col,"has invalid variables",v.0.to_string()));
                }
                variables3_checked.push(v);
            }

            let remainder_action = &new_action[variable_right_delims[0]+1..].replace(" ,",",").replace(", ",",");

            let mut delim = vec![];
            line_pos = 0;
            for c in remainder_action.chars(){
                if c == ','{
                    delim.push(line_pos);
                }
                line_pos += 1;
            }

            let compare_val = &remainder_action[..delim[0]];
            let true_val = &remainder_action[delim[0]+1..delim[1]].replace("\'C\'","\'Halcyon\'").replace("\'S\'","\'Ironclad\'");
            let false_val = &remainder_action[delim[1]+1..].replace("\'C\'","\'Halcyon\'").replace("\'S\'","\'Ironclad\'");

            println!("                if rd_fields['{}'] == {} :", variables3_checked[0].0.replace("<","").replace(">",""), compare_val);
            println!("                    attrs['ACTION'] = {}", true_val);
            println!("                else:");
            println!("                    attrs['ACTION'] = {}", false_val);
        }


        //HANDLE MERCY_EXPRESSION (Input Column #5)
        if mercy_col == "<Mercy>"{
            println!("                attrs['MERCY_EXPRESSION'] = rd_fields['mercy']");
        }
        else if mercy_col == "<2 blank spaces>"{
            println!("                attrs['MERCY_EXPRESSION'] = '  '");
        }
        else{
            println!("                attrs['MERCY_EXPRESSION'] = '{}'", mercy_col);
        }


        //HANDLE OPERATION_TYPE (Input Column #6)
        println!("                attrs['OPERATION_TYPE'] = '{}'", operation_type_col);
        println!("                matchedYet = True");

        idx = idx + 1;
    }

    println!("### End of Output");

    //Print Errors if needed
    if false{
        println!("");  
        println!("###############");
        println!("####LOG VECTOR:");
        for e in log_vector{
            println!("{:?}", e);  
            println!("{} {} {}",e.0, e.1, e.2);
        }
    }
    
}

