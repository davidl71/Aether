"""
swiftness_models.py - Data models for Swiftness (Israeli Pension Clearing House) positions
"""
from dataclasses import dataclass
from datetime import datetime
from typing import Optional, List


@dataclass
class InsuranceCoverage:
    """Insurance coverage information (Sheet 1: כיסויים ביטוחיים)"""
    coverage_type: str  # סוג הכיסוי הביטוחי
    plan_name: str  # שם התוכנית
    company: str  # שם חברה מנהלת/מבטחת
    payment_recipient: str  # מקבל התשלום
    one_time_amount: float  # סכום חד פעמי (ILS)
    monthly_benefit: float  # קצבה חודשית (ILS)
    policy_number: str  # מס' פוליסה/חשבון (Unique identifier)
    file_modified_date: datetime  # Use file mod date as validity (no validity date in sheet)


@dataclass
class DepositRecord:
    """Monthly deposit tracking record (Sheet 2: מעקב הפקדות)"""
    product_type: str  # סוג מוצר
    company: str  # שם חברה מנהלת
    policy_number: str  # מספר פוליסה (Unique identifier)
    value_date: datetime  # תאריך ערך (VALIDITY DATE)
    salary_month: datetime  # חודש שכר
    employer_name: str  # שם מעסיק
    employee_deposit: float  # הפקדות עובד (ILS)
    employer_deposit: float  # הפקדות מעסיק (ILS)
    employer_severance: float  # הפקדות מעסיק לפיצויים (ILS)


@dataclass
class ProductDetails:
    """Product/policy detailed information (Sheet 3: פרטי המוצרים שלי)"""
    product_name: str  # שם מוצר
    company: str  # שם חברה מנהלת
    policy_number: str  # מספר פוליסה (Unique identifier)
    status: str  # סטטוס (Active/Inactive)
    total_savings: float  # סך הכל חיסכון (ILS)
    next_withdrawal_date: Optional[datetime]  # תחנת משיכה קרובה
    projected_savings_no_premiums: float  # חיסכון צפוי לגיל פרישה לא כולל פרמיות
    monthly_benefit_no_premiums: float  # קיצבה חודשית לגיל פרישה לא כולל פרמיות
    projected_savings: float  # חיסכון צפוי לגיל פרישה
    monthly_benefit: float  # קיצבה חודשית לגיל פרישה
    expected_pension_rate: float  # שיעור פנסיה זקנה צפויה
    management_fee_on_deposits: float  # שיעור דמי ניהול מהפקדות
    management_fee_annual: float  # שיעור דמי ניהול שנתי מחיסכון צבור
    ytd_return: float  # תשואה מתחילת השנה
    employee_deposits: float  # הפקדות חוסך
    employer_deposits: float  # הפקדות מעסיק
    first_join_date: Optional[datetime]  # תאריך הצטרפות לראשונה
    plan_opening_date: Optional[datetime]  # תאריך פתיחת תוכנית
    data_accuracy_date: datetime  # תאריך נכונות נתונים (VALIDITY DATE)


@dataclass
class SwiftnessData:
    """Complete Swiftness data from all sheets"""
    insurance_coverage: List[InsuranceCoverage]
    deposits: List[DepositRecord]
    products: List[ProductDetails]
    file_path: str
    file_modified_date: datetime
